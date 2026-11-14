# Networking: where the time goes

An investigation (2026-11-14, Bob: "see if there's any optimization we can do to the networking to speed things
up ... not for implementation just yet"). **Nothing here is changed yet**; this page is what was measured, what it
means, and what could be done, in the order it looks worth doing.

Measured from Bob's bundle 20261114-033457 (Kawari's own captures, 47 and 14 minutes) and his Chronofoil capture
da424662 (retail, 69 minutes), with scripts in the session's scratchpad (frames and segments per direction; the time
from a client packet to the server's next send; how often an actor's moves are sent).

## What isn't slow

Handling a request. Where the client's packet has an answer, Kawari sends it within a few milliseconds: ActionRequest
0 ms median (1 ms at the 95th percentile, 17 ms at worst), StartTalkEvent and EventReturnHandler 0 to 1 ms, PingSync 0
to 1 ms, FinishLoading 2 to 4 ms. The server's work per packet isn't what a player waits on.

## What differs from retail

| | Retail | Kawari | Why it matters |
|---|---|---|---|
| Other actors' movement | an ActorMove every **313 ms** a mover (median; 9,984 of 16,385 gaps at 300-400 ms) | every **500 ms** (the logic tick, `SERVER_TICK_MILLISECONDS`) | Enemies and NPCs are placed 60 % as often: they glide longer between corrections and turn late. The most visible "slowness". |
| Segments in a frame | 2.0 on average; 38 % of frames carry several, up to 20 and more at a zone-in | always **1** (`send_segment` hands `send_packet` one) | Each segment gets its own 40-byte frame header, its own compression call and its own write to the socket. In the 47-minute capture: 5,775 frames for 5,775 segments, 226 KiB of headers on 600 KiB of segments; grouped as they were produced (within 1 ms) they'd be 3,010 frames. A login's inventory is 113 frames in 3 ms. |
| Small writes | — | `TCP_NODELAY` is never set (no `set_nodelay` in the world, lobby or login servers) | With Nagle's algorithm on, a small write waits while an earlier one is unacknowledged; a Windows client acknowledges every second segment or after up to 200 ms. One-segment frames are exactly the case it delays. How much it bites depends on the route: behind rootless Podman's port forwarder (which acknowledges at once, locally) little; with host networking or a direct route, bursts of small packets can stall 40 to 200 ms. |
| HP/MP updates to oneself | every 3 s | at irregular gaps (median 910 ms in the capture, some under 100 ms) | More packets than retail for the same information; harmless, but they add to the one-frame-each cost. |

## What could hold a connection up

Found by reading the code, not yet measured under load:

* **The capture recorder flushes after every packet** (`capture.rs`: a Debug string and a hex string built, a JSON
  line written, `flush()`), on the connection's own task, before the packet is sent. With captures on (Bob's server:
  every bundle has them) that's a write to disk per packet in the send path.
* **The world's inbox holds 64 messages** (`spawn_main_loop`: `channel(64)`); every connection's `handle.send(...)`
  waits when it's full. The world task handles one message at a time and also runs the tick (logic, navmesh paths,
  Lua for battles), so one slow step delays every player's messages behind it.
* **Game data is one mutex** for every connection and the world task, and some lookups read a sheet from the game
  files each time they're asked while holding it (`get_gc_rank_seals` and others that call `read_from` per call)
  rather than once.
* **The database is written on the connection's task** (SQLite through Diesel, synchronous): an autosave or a commit
  holds that task, and the database mutex, until the disk answers.

## What could be done, in order

1. **Set `TCP_NODELAY`** on every accepted socket (world, lobby, login). One line each, no behaviour change, removes
   the worst case outright.
2. **Send what a handler produced as one frame.** Collect the segments a connection's handler queues and write them
   together when it's done (or when a frame is full), as retail does: fewer writes and headers, better compression,
   and the client gets a batch at once instead of over several reads. This is the larger change: `send_segment` is
   called from everywhere, so the collecting has to sit under it, with a flush at the end of each handled packet,
   server message and tick.
3. **Move others every 300 ms or so**, as retail: either the logic tick itself (it also drives regeneration, auto
   attacks and gauges, which count in its 500 ms, so those would need their own timing) or movement on its own finer
   tick. Needs retail's numbers checked per kind of mover first (the capture's second cluster, 500 to 900 ms, is
   probably slow walkers and patrols).
4. **Take the capture's write off the send path**: buffer lines and flush on a timer or from another task (a crash
   then loses the last moments, which is what the per-packet flush was for; a bundle is made from a running server).
5. **Read each sheet once** where a lookup reads it per call, and look at what the world task does in one step
   (navmesh, Lua) if players report everyone freezing at once.
6. **Give the world's inbox more room**, or make sends to it not wait.

Before any of it: a measurement on Bob's server with two players in a busy fight (a capture from each, and the world
task's time per step) would say which of these is felt.
