# Saving and connections

When a character's progress reaches the database (`world.db`), and what happens when a connection drops.

## When the server saves

| When | What is saved | Where |
|---|---|---|
| **Every 30 s** while in the world (`AUTOSAVE_INTERVAL`, `main.rs`) | Everything logging out saves (`commit_player_data`), including playtime, but the player stays online. | `ZoneConnection::autosave()` (`zone_connection/mod.rs`) |
| Zone change | Only the volatile row: zone, position, rotation (as upstream). | `zone_connection/zone.rs` (`commit_volatile`) |
| Class/job change | Class levels and inventory. | `zone_connection/stats.rs` (`commit_classjob_and_inventory`) |
| Logout (graceful or forced) | Everything; marks the player offline; adds the session to playtime. | `begin_log_out()` |

Inside an instance (condition `BoundByDuty`) autosave and logout save the zone and position the player **entered from**,
so they never log back into an instance that's gone.

Saving everything on zone change was tried (`d543ce9`) and coincided with zone changes loading forever; it and the
"zone + quests" save that replaced it (`4070c7c`) went away with the quest rollback (`3fb25fe`). The 30-second
autosave covers quests and everything else.

## Dead connections

A connection that drops without closing (network gone, PC off) never reads 0 bytes, so upstream waited on it forever:
the player was never logged out or saved until the server stopped. The zone connection now drops itself after
**60 s of silence** (`DEAD_CONNECTION_TIMEOUT`, measured from the last keep-alive). The client pings every ~10 s even
on loading screens. Dropping logs the player out normally (forced logout), which saves them.

Worst case after a sudden disconnect: the last ≤30 s of progress is lost.

## Message queues

Each zone connection has a message queue from the server. It used to hold 64; when full, every send failed and the
server silently removed the player from the world (monsters froze, nothing could be attacked until relog). Zone and
chat queues now hold 4096; a full queue drops that one message with a warning, and only a **closed** connection gets
the client removed. Removals are logged.

## Playtime

* `playtime_at_login` is set from the database at login; `playtime_minutes()` = that + minutes since login.
* Autosave, logout and `/playtime` all use it, so a session is never counted twice.
* Because autosave writes `time_played_minutes`, Kawari Web's play coins (earned from playtime) keep up while people
  play. See [Kawari Web](kawari-web.md#shop).

## Other things saved by the fork

* **Map discovery**: each map part the client discovers is set in the character's discovery bitmask (Map sheet
  `DiscoveryIndex`/`DiscoveryArrayByte` say where), saved in `unlock.map_discovery` (migration
  `update-2026-10-14_map_discovery`, size `MAP_DISCOVERY_BITMASK_SIZE` in `constants.yml`) and sent in PlayerSetup.
* **Gear swapped from the main inventory** goes to the Armoury Chest, as the client expects (they used to disagree,
  and the piece was lost on next login).

## Editing a character from outside

The server keeps a logged-in character in memory and saves over the database every 30 s and at logout. So anything
that edits `world.db` directly (Kawari Web's gil purchases, Quest Lab's character setup) must only do it while the
character is **logged out**. Mail is the exception: the server reads the `mail` table each time the mailbox opens.
