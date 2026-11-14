# Stability: what takes the server down, and what's kept to one player

An audit (2026-11-14, Bob: "do your due diligence in making sure the server is just as stable as it is fast") of
every `server.log` in the diagnostics repository (248 bundles, 2026-10-13 to 2026-11-14) for panics, dropped
connections, streams out of step and stalls, and what was built from it. **None of it has been played yet.**

## What the logs say

| What | How often | What it did | Status before this change |
|---|---|---|---|
| A panic in the world server | 10 in 10 bundles, from 4 places (`fate.rs:54`, `actor.rs:174` six times in a morning, `actor.rs:181` twice, `bitmask.rs:117`) | **The whole server ended**: the release profile aborts on a panic, and it's the last line of each of those logs. `kawari-run` waits for each server once, so the world stayed down, with the lobby and login still up, until the container was restarted. | Each bug fixed as it was found; nothing stopped the next one doing the same. |
| A client's stream out of step (Oodle) | 5 bundles (2026-10-16 three times, 2026-10-30, 2026-11-10), up to 316 unreadable packets each | The client froze or loaded forever: nothing it sent could be read again, and it wasn't told. | Two causes fixed (a slow sheet read; a read holding two packets, `take_packets`); the failure itself left as it was. |
| The world's tick holding the turn | 3 times on 2026-11-14, each right after a start, 45 s | No one could log in for 45 s after every start or deploy: the first tick made 927 world bots at 45 ms each. | Logged by the watchdog; not looked into. |
| A connection opened and closed with nothing sent | 44 | Two error lines each (the ControlPanel's port check). | Noise among the errors that matter. |
| "timed out" drops | 14 zone, 100 chat, 389 lobby | The ordinary end of a client that closed, or of a stream out of step (above). | — |

The client's own crashes (the Stone Vigil's exit on 2026-11-07, the teleport of 2026-10-27, the battle talk, the FATE
director) are in none of these logs: the server sees a connection end. Every one traced so far was a packet the
client couldn't take; those are fixed one at a time, from the bundle and the client's crash log, and this page
doesn't change that.

## What was built

### A panic ends what it happened in (`servers/world/src/contain.rs`)

The release profile unwinds (`Cargo.toml`: `panic = "unwind"`, upstream's is `"abort"`), and:

* **In a connection** (handling a client's packet, a message from the world, a timer): the connection ends as a lost
  one does. `client_loop` runs the turns (`client_turns`) inside `contain::Contained`, and whether they ended or
  panicked, `client_ended` logs the player out and saves them and tells the world they left. What was waiting to be
  sent is dropped. The player sees the connection lost and logs back in; no one else notices. Should the saving
  itself panic, `start_client` still tells the world (the character then isn't saved past its last autosave, 30 s).
  The chat connection the same. A connection's first packets (`initial_setup`, which panics on a first segment it
  doesn't expect, from anyone who can reach the port) end only that connection, as any tokio task does now.
* **In a step of the world** (one message from a connection, or one tick): `contain::step` logs it and the world goes
  on to the next step. The step's locks are released as it unwinds (parking_lot's don't poison); what the step was
  half way through stays half done, which is the risk taken instead of everyone being dropped. For this the message
  loop's body and the tick's are closures (`handle_message`, `tick` in `server/mod.rs`): what skipped to the next
  message with `continue` now returns (55 places; a new `continue` at that level doesn't compile, so it can't be
  missed in a merge), and `FatalError` is carried out through `fatal`.
* **A storm**: more than 20 panics in 10 s (the same broken state every tick) ends the process (exit 101), to be
  started again fresh.

Every one is logged as an error with the panic's message, after the default hook's location and backtrace. They're
bugs to fix as before; a bundle now shows them without the server having died.

### `kawari-run` starts a server again (`tools/run/src/main.rs`)

Each of the seven servers is started again when it ends, whatever the reason: after 1 s if it had run two minutes, otherwise 2 s doubling to at most 60 s
while it keeps ending within two minutes of starting (`next_wait`), and quick again once one has run that long. A
crash is then a restart's outage, not one that lasts until someone looks.

### A stream out of step is dropped (`contain::out_of_step`)

Three of a client's packets in a row that should hold segments and give none: the connection is dropped (and the
player saved), so the client shows the connection as lost at once and can log back in, instead of waiting frozen. One
unreadable packet in a stream that reads again doesn't count. Why a busy moment loses the place in the stream isn't
known any better than before; the [networking](networking.md) change shortens the moments.

### A world stuck for three minutes ends itself (`world_turn.rs`)

The watchdog still says after 15 s that a step is holding the turn; at 180 s (`GIVE_UP_AFTER`) it ends the process
(exit 102) for `kawari-run` to start again. A step that long is a loop that won't finish; characters are as their
connections last saved them (the 30 s autosave).

### Making a bot no longer reads the game's files (`party_bot.rs`, `gamedata.rs`)

Each bot made (world bots at the start and when a marketboard's zone is first seen, a Duty Finder party's, a Duty
Support party's gear) read the whole Item sheet (`best_gear`), EquipRaceCategory (`wearable_restrictions`),
ClassJobCategory (`classjob_categories_with`) and CharaMakeType with CharaMakeCustomize (`creator_looks`), with the
world's turn held. They're read once now and kept on `GameData` (`bot_gear_rows`, `bot_wearable`,
`bot_class_categories`, `bot_creator_looks`; nothing is kept when a sheet can't be read, so it's tried again). The 45 s
at each start should be about a second; **it needs Bob's server to say so** (the game's files aren't in the test
environment), by the "World bots: N made" line's time after "World bots: on".

### Quieter

A connection that sends nothing before closing isn't parsed (`initial_setup`'s `Ok(0)`).

## Tests

`contain::tests` (a panicking step is dropped and the next runs; its locks are free afterwards; a storm; a
connection's panic comes back to its owner with what it borrowed usable; a stream out of step against one bad
packet), `kawari-run`'s `next_wait`. By hand: the release profile builds (`kawari-world`, `kawari-run`), and that
`kawari-run` with stand-in servers started a "world" that exits with 101 again after 2 s, then 4 s. Not covered: a
panic in a real handler end to end in a release build, and the bots' start-up time (both need the game's files).

## What to look for in a bundle now

* `The connection of ClientId(N) (name) panicked:` one player dropped by a bug; the panic's location is just above.
* `The world server panicked handling a message|the tick:` a step dropped; the world went on.
* `kawari-run: kawari-world ended (...)` the server ended and was started again.
* `Dropping zone connection from ...: 3 packets in a row couldn't be read` a stream out of step.
