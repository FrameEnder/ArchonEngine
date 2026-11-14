# Networking: where the time goes

An investigation (2026-11-14, Bob: "see if there's any optimization we can do to the networking to speed things
up ... not for implementation just yet"), then its findings built the same day ("OK implement your network
findings"): [what was built](#what-was-built-2026-11-14) is at the end, with the one that wasn't (others' movement
every 300 ms) and why. The page first says what was measured and what it means; its tables describe the server
**before** the change.

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

As proposed; 1, 2, 4, 5 and 6 are built (below), 3 isn't.

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

## What was built (2026-11-14)

None of it has been played yet: built, unit-tested, and waiting for Bob's server.

| # | What | Where |
|---|---|---|
| 1 | `TCP_NODELAY` on every game connection the world and the lobby accept. The login server is HTTP (axum), not a game connection, and is left alone. | `servers/world/src/main.rs`, `servers/lobby/src/main.rs` (after `accept`) |
| 2 | **What one turn of a connection's loop sends is one frame.** `send_segment` queues (`OutgoingFrame`) while the loop runs (`ZoneConnection::batching`); the loop sends the frame at the top of its next turn (`flush_segments`), so a client packet's answers, a server message's packets, or a timer's go out together, in order. A frame stops at 16,000 bytes (`FRAME_BYTES`, retail's largest in da424662 was 16,312) and the rest starts another. Sent first, so order holds: before a keep-alive's reply (it writes to the socket itself), before new obfuscation keys are made at a zone change, and after `InitializeObfuscation` (what follows is written with the keys it announces). Before the loop starts and after it ends, segments go one a frame as before. | `servers/world/src/zone_connection/mod.rs` (`OutgoingFrame`, `send_segment`, `flush_segments`), `zone.rs`, the loop in `main.rs` |
| 4 | **The capture's write is off the send path.** A packet's line waits in the writer's buffer; the loop writes it out when 250 ms have passed (`capture::FLUSH_INTERVAL`, `flush_if_due`), or after that long with nothing else happening (a branch of the loop's `select!`). A note (`!mark`) is written at once with what waited, and everything when the connection ends. A bundle made from a running server can miss a connection's last 250 ms; a server that's killed, the same. | `servers/world/src/capture.rs`, the loop in `main.rs` |
| 5 | **Sheets read once** for the lookups that read one per call on the world's task or often: EventItem, ItemAction (using an item), EventAction (an event's cast bar), Emote, NpcYell, QuestBattle, ContentDirectorManagedSG (a duty's map effects) and GrandCompanyRank (seals). `GameData::kept_sheet` reads a sheet the first time it's asked for and keeps it. About seventy other lookups still read per call (shops, aetherytes, recipes, housing, the Gold Saucer: once per click, on the connection's task); they take `kept_sheet` the same way when one shows up as slow. | `servers/world/src/gamedata.rs` |
| 6 | The world's inbox holds 4,096 messages (`CLIENT_QUEUE_SIZE`), not 64. | `servers/world/src/main.rs` (`spawn_main_loop`) |

Tests: `zone_connection::outgoing_frame_tests` (a handler's segments are one frame in order; a full frame goes
first), `capture::tests::lines_are_written_out_on_a_timer_not_per_packet`.

### Not built: others' movement every 300 ms (3)

Measured further first, as this page said it should be (both of Bob's captures at hand, da424662 and d1c46dff, gaps
between one mover's ActorMoves, `net/moves2.py` in the session's scratchpad):

| Mover | Gaps counted | Median | 10th to 90th percentile | Where they fall |
|---|---|---|---|---|
| Battle NPCs | 26,846 | 313 ms | 311 to 891 ms | 16,653 at 300-350 ms; 4,766 at 550-650; 4,059 at 850-950 |
| Other players | 925 | 355 ms | 268 to 891 ms | 497 at 300-400 ms; the client's own sends relayed, so less regular |

So retail places a moving NPC about every 313 ms, and at two and three times that for some (slow walkers, or a step
with nothing new to say). Kawari's is 500 ms because movement is a part of the behaviour tick: `npc_behavior` (one
function: noticing, choosing a target, paths, reach, timelines, casts) advances a path by
`speed * BEHAVIOR_TICK_SECONDS` and sends the ActorMove in the same pass, and party bots, world bots and companions
step the same way. The tick's 500 ms is also what auto-attacks, regeneration, gauges, rested EXP and battle timelines
(`timeline_position * BEHAVIOR_TICK_SECONDS`) count in.

Changing the tick's length changes all of those; sending an extra ActorMove between ticks without moving the actor on
the server would be a workaround that the next mechanic has to know about. Doing it as retail does means taking
movement out of the behaviour pass into its own step (every 300 ms or so, by time elapsed, for NPCs, bots and
companions alike), with the behaviour pass only choosing where to go. That's a rebuild of the fork's most-tested
code (every duty's `duty-sim`, the party bots), and it shouldn't go in unplayed alongside a change to how every
packet is sent: if something then looked wrong in game, there'd be no telling which did it. It's in
`known-issues.md` to do next, by itself, once the framing change has been played.

## Still to measure

A capture from Bob's server after this change (frames aren't in Kawari's capture, only segments; the client's feel
and Chronofoil on the client's side say more), and with two players in a busy fight, the world task's time per step:
the database written on the connection's task, and the one game-data mutex, are untouched.
