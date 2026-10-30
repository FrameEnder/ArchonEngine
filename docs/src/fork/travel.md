# Travel

Aetherytes, attunement, Teleport and maps.

## Attuning

`resources/scripts/events/generic/Aetheryte.lua` and `AethernetShard.lua`:

* Talking to an aetheryte or shard you haven't attuned to starts **EventAction 19** ("Attuning", 5 s bar and
  animation, Sapphire's `ACTION_ATTUNE`) instead of unlocking it instantly.
* When the bar completes (`onEventActionCast`) the aetheryte is unlocked (`unlock_aetheryte`). Shards then play scene 3
  ("you have aethernet access" and its effect, flags `0x2001` and params `{1, 1}` as retail: Bob's 2026-10-18 capture of a Limsa Lominsa shard; the attunement itself, EventAction 19 for 5 s then LearnTeleport, already matched); main aetherytes end the event.
* Attuning while Ul'dah's Close to Home asks for it nests the aetheryte's event in the quest's and plays the same
  bar, then the aetheryte's notice 0 and the quest's scenes (quests, "Attuning inside a quest").
* Already attuned: the aetheryte menu (scene 2) as upstream.
* Close to Home's attunement objective uses the aetheryte's own event, and also teaches Return.

## The aethernet's other destinations

Attuning to the last of a city's aethernet shards also opens the aethernet's other destinations: the city's gates and
its airship landing, which the aethernet menu then lists. They are the Aetheryte sheet's rows with `Invisible` set in
the city's `AethernetGroup`; the group's visible shards (not `IsAetheryte`, so the city's big aetheryte isn't needed)
must all be attuned. A destination with a `RequiredQuest` waits for that quest too (row 54, the White Wolf Gate, for
quest 66338).

Retail (Bob's Chronofoil capture cc62ad94, 2026-10-28, all three cities) sent a LearnTeleport (ActorControlSelf 509)
for each of them right after the last shard's, in the same burst; the character's aetheryte bitmask at its next login
(PlayerSetup: the unlocks' 92 bytes start 64 bytes after the name, the aetherytes' 30 follow) had them all set.

`GameData::aethernet_extras` lists each group's shards and destinations; `ZoneConnection::learn_aethernet_extras`
(`zone_connection/unlock.rs`) teaches any that are due, after a shard is attuned (`LuaTask::UnlockAetheryte`), after a
quest is completed (for `RequiredQuest`) and at login (so older characters get theirs.)

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

**Warp type.** Retail's Teleport is warp type 4 (Return 7) in both PrepareZoning and, within a zone, ActorSetPos, with
the character hidden (`hide_character` 1) and flags 0x02 within a zone, 0x0A (with PRESERVE_COMPANION) to another one:
all 15 teleports in Bob's Chronofoil capture of 2026-10-21. Kawari sent warp type 1 with nothing hidden, and a teleport
within the zone you're in (Moraby Drydocks from Lower La Noscea, Bentbranch from the Central Shroud) left the player in
the teleport animation after landing (2026-10-16 and 2026-10-22 bundles). Since 2026-10-22 the Teleport spell passes
`player:warp_aetheryte(id, false, true)` and Return uses warp type 7; aethernet shards and housing exits keep type 1.
Retail also sends ActorControlSelf 263 (1 as it starts, 0 after landing), 236 after the ActorSetPos and 253 with 0, 1
after landing; Kawari doesn't yet.

**Teleport costs** (`GameData::teleport_cost`), as the client's Telepo GetTeleportCost works them out and exact for
all 87 paid teleports in Bob's Chronofoil captures: each zone has a point on a world map (TerritoryTypeTelepo: X, Y,
Expansion, Relay). A leg between two points costs `floor(distance × m / 1000) / 5 + 100`, where the distance's x wraps
around the region's width (TelepoRelay's last value) and `m` is 1000, or the two Expansion values plus 600 once they
add up to 400 or more. In the same region it's one leg; across regions TelepoRelay gives the crossing (enter zone,
exit zone, cost): the cost plus a leg to the crossing and one from it. Above 1000 it's 1000 plus half the rest; a
favourite destination costs half, the free destination nothing (the character's saved favourites and free
aetheryte); housing destinations a quarter (not done). Sapphire's `distance / 2 + 100` was wrong. Kawari charged
nothing before. As retail, the gil is taken when the cast lands (the gil slot, then LogMessage 4590 with the cost);
a teleport the character can't afford is refused at once (TeleportStart with 2, Sapphire's value).

## Leaving a zone with FATEs

Retail takes the zone's FATEs down before the player loads the next map (Bob's Chronofoil captures, leaving the
Central Shroud): right after PrepareZoning, actor control **2352** (`UnkFate2`) with each FATE start NPC's actor ID,
then **2357** for every FATE the zone has (about 45 IDs, running or not; the fork calls it `RemoveFateContext`), then
TerminateDirector for the FATE director (handler `0x801AFFFF`). Entering, it sends InitDirector for that director right
after ZoneInit and CreateFateContext for each FATE during the loading screen, and FateInit and the targetable status
once the player has loaded in.

Kawari only sent the TerminateDirector, so the client kept its FATE contexts with no director and crashed in its FATE
update on the next frame (2026-10-17, teleporting from the Central Shroud to Gridania: an access violation writing
0x430 into a null FateDirector, in FateContext code under `FateManager` and `EventFramework::Update`, named from
FFXIVClientStructs' `ida/data.yml` for this patch). `remove_actor_from_instance` now sends 2352 and 2357 first, in
retail's order.

## Warps and their conditions

A warp (a Warp row: lifts, doors, ferries, airship ticketers) may ask something of the player, its WarpCondition: quests
done (all of them, or any by CompleteParam 2), a class level and a fare in gil. The generic `events/generic/Warp.lua`
reads it through `player:warp_info(id)` (`GameData::get_warp_details`) and plays it as retail did:

* **Met**, with a ConditionSuccessEvent: that DefaultTalk first, nested (EventStart type 7, its scene 0, 0x2000), then
  the yes/no (Aleport's ferry, warp 37, in Bob's Chronofoil capture cc62ad94; the lifts to and from Chocobo Square, 105
  and 106, in deaf7e28.)
* **Not met**: the ConditionFailEvent instead of the yes/no, and nothing more. The client itself refuses the yes/no then
  (error code 50, at once): Bob's bundle 20261029-154412, the Ul'dah ticketers (warps 29 and 30, 120 gil) with too
  little gil, and the Gold Saucer's (108) before It Could Happen to You. Retail's fail talk wasn't captured; it's
  played as the success talk is. Before 2026-10-29 Kawari played the yes/no regardless and nothing came of it.
* **Yes** on a warp with a StartCutscene (the airships) plays scene 1000 (retail's flags 0x482402, the flight from
  Limsa Lominsa to Ul'dah, warp 31), then warps. The **fare** is taken as the warp goes (retail's gil slot came with
  PrepareZoning: Aleport's 40 gil, the airship's 120); Kawari took none until 2026-10-29 (Bob's bundle
  20261029-154625.)
* **The class level** is the player's current class's (the synced one while synced). Event scripts saw level 0 until
  2026-10-30 (`LuaPlayer::current_level` was only filled in for actions), so every warp asking for a level turned the
  player away: Romarique's ferry to Sweetbloom Pier (warp 131077, WarpCondition 4: level 10, no quests or fare)
  played its fail talk (DefaultTalk 590355) to a level 24 (Bob's bundle 20261030-101331). The connection now fills it
  in with the player's data whenever it refreshes the scripts' view (`ZoneConnection::sync_lua_player`).

## Warping within a zone

A warp that stays in the zone (an aetheryte shard, a door in the Waking Sands) keeps the player in the instance
they're in (`begin_change_zone`). Before 2026-10-28 a warp within a private area (the Waking Sands, 212) made a new
instance of it every time, and the player stayed in the old one too; the next zone change took them out of one of
the two, so they were in two instances, and the zone loading after that picked whichever came first. In Bob's bundle
of 2026-10-28 that sent Lord of the Inferno's quest battle (275) the Eastern Thanalan instance he had left: its 16
monsters (the invisible city's zombies attacking during the loading screen), its FATEs, the FATE director instead of
the battle's, no opening, and the player dead 81 s in. Before that, in Eastern Thanalan itself, his position
updates went to the copy left in the Waking Sands, so the server kept him where he had arrived (36, 1, 1) and never
sent the monsters he walked past: the invisible city's nine zombies (bases 18 and 19) never appeared, though the
instance had them. `remove_actor_from_instance` now also takes a player out of
every instance they're in.

## Chocobo porters

A chocobo porter (a ChocoboTaxiStand, handler type 18: events `0x0012xxxx`, the stand being the low 16 bits) takes the
player along a set route to another stand, the client riding it on its own. Kawari plays it as retail did in Bob's
Chronofoil captures cc62ad94, 1fac2416 and 2528a32d (nine rides: ChocoboTaxi 25 in the South Shroud, 56 and 57 in
Eastern Thanalan; client 2026.10.16). `events/generic/Chocobokeep.lua` is the porter, `zone_connection/chocobo_taxi.rs`
the ride.

* **The first talk registers the stand**: ActorControlSelf 510 (`ToggleChocoboTaxiStandUnlock [stand, 1]`) at the
  EventStart, then scene 0 with `[1]` (flags 0x2000), whose answer means nothing; no ride. Some stands registered
  another along with them: 5 and 3 with 2, 24 with 21, 15 with 10; 4 and 19 alone. Why these isn't known, so they're
  a table in the script (`ALSO_REGISTERS`). `player:has_chocobo_taxi_stand(id)` reads the player's stands.
* **Later talks** play scene 0 with `[0]`: the menu, whose first result is the ChocoboTaxi row chosen (25, 56, 57), or
  0. `player:ride_chocobo_taxi(taxi, x, y, z, rotation)` starts the ride after the EventFinish.
* **The ride**: the fare (ChocoboTaxi's Fare) is taken from the gil slot; Condition BetweenAreas; PrepareZoning to the
  same territory with warp type 10 (ChocoboTaxi), the character hidden and flags 0x02 (`warp_appearance`); about 1.5 s
  later ActorSetPos (warp type 10, arg 2) at the route's start. Retail's starts were a few yalms from the porter and
  the same every time: stand 5 (174.14, 8.19, -52.94) facing -1.521, 15 (-432.33, -38.6, 99.95) facing 2.995, 19
  (-538.72, -0.35, -200.45) facing -0.238 (`ROUTE_STARTS`); other stands start where the player stands.
* **On the chocobo**: SetMode mounted (ActorControl 2 `[4]`), the Mount packet (mount 3, everything after it 0, to
  everyone around: `ToServer::ChocoboTaxiMount`), the online status UsingAChocoboPorter (16), and `ChocoboTaxiStart`
  (server opcode 538, [Fork opcodes](opcodes.md)): the ChocoboTaxi row and its Location column (25 and 3792350, 56
  and 3686766, 57 and 4011314). Then UsingChocoboTaxi (22) instead of BetweenAreas. Retail sent all this right after
  the ActorSetPos, before the client's FinishZoning (ClientTrigger 201 `[0, 10]`), and the Condition after it; Kawari
  sends it all as the client finishes zoning, since the ActorSetPos is a queued task.
* **On the way** the client sends `ChocoboTaxiCheckpoint` (client opcode 534: the row and the point's number, 1, 2,
  3…) every few seconds, its position updates as usual, and at the end `ChocoboTaxiArrived` (518: the last point's
  number, 13 for ChocoboTaxi 25, 11 for 56, 14 for 57). The rides took 43 s (25) and 59-60 s (56, 57). Arrivals were
  around (-196.2, 8.69, -56.6) for 25, (-539.7, -0.6, -198.1) for 56 and (-434.0, -38.5, 96.9) for 57, each a few
  yalms from the other end's porter.
* **At the end**: the online status back, SetMode normal (ActorControl 2 `[1]`), the Condition cleared.

Not sent (what they do isn't known): ActorControlSelf 263 `[1]` with the fare and `[0]` at the end, 236 `[1]` and
0 `[0, 1, 0]` with the ActorSetPos, 928 `[9]`, 504 `[32]` at the start and end, 930, 253 `[0, 1]` and 260
`[0, 0, 0, 7]` at the end; and the EventFinish's result byte, 0x12 for the ride on retail (Kawari's 1).

## Map discovery

Discovered map parts are saved, sent back at login, and A Realm Reborn's field zones are revealed whole on the first
visit (see [Achievements](character-windows.md) and [Saving](saving.md)).

## The Gridania opening area

The opening takes place in a separate copy of New Gridania (territory 183) with guards and invisible walls. Accepting
**Close to Home** moves the player into the real New Gridania (132, the quest's `TERRITORYTYPE0`). Earlier attempts to
move players out of 183 at login, or to drop the guards' boundaries once the opening is done, were rolled back with
the first quest engine (`3fb25fe`); only the Close to Home move remains.

## Layer filters (the Waking Sands)

A zone's layout (its LVB) can list several layer filters (physis' "layer sets": an ID, a territory and a duty), and
each of its layers shows only under some of them (a Match or NoMatch list of keys; FFXIVClientStructs'
`FileLayerGroupLayerFilter`, `LayoutManager.LayerFilterKey`). Where the story changes a place, retail's server tells
the client which filter to use, by quest progress: ZoneInit's `transition_territory_filter_key` as you come in, and
ActorSetPos' as a quest's cutscene moves you within the zone. In the Waking Sands (Bob's Chronofoil capture afe32479
visit 0.10, 2026-10-25) it was 90338 at The Scions of the Seventh Dawn's last step and after it was done, and 90339 in
the ActorSetPos as A Wild Rose by Any Other Name (510) was accepted, and after; every other zone in the captures had 0.
Later (Bob's capture b8432613, 2026-10-27): 90339 through Lord of the Inferno (343), 90340 as A Hero in the Making
(511) was accepted (its opening cutscene's ActorSetPos), 90341 once Wood's Will Be Done (683, the Twin Adders' next
step; the other companies' 684 and 685 the same: 685's confirmed in Dags' run, Bob's Chronofoil capture c99fe374 of
2026-10-28, 90341 in ZoneInit once it was accepted and 90342 as Sylph-management was; 684 is taken to be) was accepted, 90342 as Sylph-management (513) was
accepted.
Kawari sent 0, and the client showed the people of every visit to the Waking Sands at once (Bob's screenshot of
2026-10-26: Thancred, Biggs, Wedge, Cid, Yda, Papalymo, Arenvald, Iliud and a row of soldiers in the hall).

Kawari reads the key from `resources/data/layer_filters.json` (`layer_filters.rs`: per territory, the last entry whose
quest has been accepted or completed; an entry without a quest is the default), puts it in ZoneInit, and fills it in
any ActorSetPos to the player that has none (`ZoneConnection::send_ipc_from`). Retail picks it on its own servers, so
each part of the story's key has to come from a capture of it; a zone with more than one filter logs them and which
of its layers each shows, with their event NPCs, the first time it's loaded (`log_layer_filters`), which is how the
rest can be matched to the story. `transition_territory_filter_key` was thought to choose ambient sound.

**Where else** (every ZoneInit and ActorSetPos in Bob's 54 Chronofoil captures, 2026-06 to 2026-10, 110 zones): in A
Realm Reborn only the Waking Sands; later, territory 351 (247671, then 251230 in later captures), 433 (119936), 844
(254682), 857 (242178, 246488), 878 (243293, 244188, 244190, 244391, 251232 across visits) and 1165 (276017 as it's
entered, then 276018 to 276022 in turn within one visit, an event moving on). Which quest each goes with isn't
recorded, so they aren't in `layer_filters.json` yet. Every other zone, cities and inns among them, had 0.

## Weather

Kawari forecasts weather like the client (WeatherRate's chances against a roll for each 8 Eorzean hour window,
`weather_target`): in integers with wrapping shifts and a strict `<`, which matched retail's natural weather in 261 of
262 zone-ins and 423 of 425 changes in Bob's Chronofoil captures (the rest were scripted duty weather). The old float
version was only exact to 128 s at today's times and used `<=`, and disagreed 8.6 % of the time. Out in the world the
weather now moves on at each 1400 s boundary (retail: 161 of 161 within 3 s), sent as WeatherId with retail's 20 s
fade; before, an instance kept the weather it started with. (Retail also re-sends the weather as a WeatherId after
every zone-in; Kawari doesn't.) EorzeanTimeOffset was never sent on retail: clients work Eorzean time out from the
server's.
