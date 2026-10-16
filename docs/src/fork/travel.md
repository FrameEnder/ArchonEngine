# Travel

Aetherytes, attunement, Teleport and maps.

## Attuning

`resources/scripts/events/generic/Aetheryte.lua` and `AethernetShard.lua`:

* Talking to an aetheryte or shard you haven't attuned to starts **EventAction 19** ("Attuning", 5 s bar and
  animation, Sapphire's `ACTION_ATTUNE`) instead of unlocking it instantly.
* When the bar completes (`onEventActionCast`) the aetheryte is unlocked (`unlock_aetheryte`). Shards then play scene 3
  ("you have aethernet access" and its effect); main aetherytes end the event.
* Already attuned: the aetheryte menu (scene 2) as upstream.
* Close to Home's attunement objective uses the aetheryte's own event, and also teaches Return.

## Teleport unlock

On retail the Teleport general action becomes usable after attuning to a **second main aetheryte** (shards don't
count). Upstream registered aetherytes (`LearnTeleport`) but never set Teleport's unlock link.
`check_teleport_unlock` (`zone_connection/zone.rs`) reads the unlock link from the GeneralAction sheet (the general
action whose action is Teleport, 5; Sapphire calls it unlock 4), sets it and tells the client (`ToggleUnlock`) when a
main aetheryte is attuned and two or more are; it also runs at login, so older characters get it.

## Teleporting

Teleport's action script queues the warp as a Lua task when the cast ends. Those tasks now run as soon as they reach
the connection (see [Timing](timing.md#tasks-from-lua)); before, the warp went out at the client's next packet, up to
~10 s later, leaving the player hovering in the teleport animation.

## Map discovery

Discovered map parts are saved and sent back at login, so maps stay revealed (see [Saving](saving.md)).

## The Gridania opening area

The opening takes place in a separate copy of New Gridania (territory 183) with guards and invisible walls. Accepting
**Close to Home** moves the player into the real New Gridania (132, the quest's `TERRITORYTYPE0`). Earlier attempts to
move players out of 183 at login, or to drop the guards' boundaries once the opening is done, were rolled back with
the first quest engine (`3fb25fe`); only the Close to Home move remains.
