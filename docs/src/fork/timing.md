# Timing, casting and tasks

How the world server schedules things, and what the fork changed so casts, progress bars and teleports land when
retail's do.

## The two loops

| Loop | Interval | Constant (`servers/world/src/server/mod.rs`) | What runs |
|---|---|---|---|
| Logic tick | 500 ms | `SERVER_TICK_MILLISECONDS` | NPC AI, auto-attacks, DoT/HoT ticks, director `onUpdate`, spawns. Unchanged from upstream. |
| Task tick | 20 ms | `TASK_TICK_MILLISECONDS` | Queued tasks whose time has come: finished casts, event action progress bars, despawns. |

Before `9e40e38` queued tasks were only checked on the logic tick, so a cast resolved up to 500 ms after its bar ended
and slidecasting was impossible. Anything that must happen "when the bar ends" belongs in a queued task
(`QueuedTaskData` in `server/instance.rs`), not in the logic tick. Things measured in logic ticks (director scripts
counting "updates", auto-attack timers) are half-second steps.

## Casting

* An action with a cast time queues a `CastAction` task at `now + adjusted cast time` (upstream's adjusted cast times,
  with Swiftcast/Presence of Mind on top), minus 0.5 s for a player's actions: retail's land half a second before the
  bar ends ([Combat](combat.md)).
* **Slidecasting**: moving cancels an interruptible cast until it lands (`task.point > now`); as it lands half a
  second early, moving during the last half second doesn't interrupt it, as on retail. Mount and item casts land at
  the end of their bar (a mount's is retail's 1.0 s Mount, action 4, and its Mount packet 0.57 s later: see
  [Chocobo companion](chocobo.md#riding)) and keep a half-second window (`task.point > now + 500 ms`).
* **Duty clock**: once a duty's countdown starts its time left is re-sent every 300 s (`resync_duty_clock`), as
  retail. Retail's wipe: PartyWipe (0x40000005), 3 s later 0x8000000D, 0x80000029, 0x40000013, then 0x4000000F (time
  left) and 0x40000011, and DutyRecommence (0x40000006) 10.2 s after the wipe. Kawari plays it out as the Bowl of
  Embers' wipe did (`server/wipe.rs`, [Dungeons](dungeons.md#the-bowl-of-embers)).
* NPC casts keep an `NpcCast` record (also where an area attack was aimed); interrupting a cast clears it.

## Event actions (progress bars)

`player:event_action(id)` (`LuaTask::EventAction` → `zone_connection/lua.rs`):

1. The client is sent ActorControl `EventAction { id }`, which shows the bar and animation (Searching, Aiding,
   Interacting, Attuning…, the EventAction sheet has the cast time).
2. For quests, the connection sends `ToServer::QuestEventAction`; the server queues `QueuedTaskData::QuestEventAction`
   for when the cast time is up (exactly then, on the task tick).
3. When it fires, the connection ends the bar and calls the script's `onEventActionCast(player)`.
4. **Interruption**: when an enemy damages the player (single-target hit, or an area attack catching a bystander,
   `server/action.rs`), `instance.interrupt_event_action` cancels the task and sends
   `FromServer::QuestEventActionInterrupted`; the connection ends the bar and finishes the event. The player tries again.

Generic aetheryte/shard attunement (EventAction 19, 5 s) uses the same bar.

## Tasks from Lua

Lua calls don't act directly: they push `LuaTask`s which the zone connection runs afterwards
(`zone_connection/lua.rs`, `process_lua_player`). Tasks queued by **action scripts** (Teleport's warp, for one) reach
the connection as `FromServer::NewTasks`; the fork runs them **immediately** (`955b0ce`). They used to wait for the
client's next packet, which standing still is the next ping, up to ~10 s: the "hovering in the teleport animation"
bug. This only affects the player whose action queued the tasks, so it doesn't speed anything else up and is safe with
many players teleporting at once: each connection runs only its own tasks.
