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
  with Swiftcast/Presence of Mind on top).
* **Slidecasting**: moving cancels an interruptible cast only if more than `SLIDECAST_WINDOW` (500 ms) remains, i.e.
  `task.point > now + 500 ms`. Moving during the last half second doesn't interrupt it, as on retail.
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
