-- Chocobo porters (ChocoboTaxiStand, handler 0x12xxxx): the first talk registers the stand, then the menu of routes,
-- and the ride (`servers/world/src/zone_connection/chocobo_taxi.rs`, `docs/src/fork/travel.md`). As retail did it in
-- Bob's Chronofoil captures cc62ad94, 1fac2416 and 2528a32d (nine rides, client 2026.10.16.)

-- Scene 0: with {1} the registration (its answer means nothing), with {0} the menu, whose answer is the ChocoboTaxi
-- row chosen, or 0.
SCENE_00000 = 00000

-- Stands registered along with another: retail's ActorControlSelf 510 unlocked both at the first talk (5 and 3 with
-- 2, 24 with 21, 15 with 10; 4 and 19 alone). Why these isn't known.
ALSO_REGISTERS = {
    [3] = 2,
    [5] = 2,
    [15] = 10,
    [24] = 21,
}

-- Where retail put the player for each stand's rides (ActorSetPos after the PrepareZoning: the route's start, a few
-- yalms from the porter). Stands not recorded start where the player stands.
ROUTE_STARTS = {
    [5] = { 174.14, 8.19, -52.94, -1.521 },     -- the South Shroud, ChocoboTaxi 25 (cc62ad94)
    [15] = { -432.33, -38.6, 99.95, 2.995 },    -- Eastern Thanalan, ChocoboTaxi 56 (1fac2416, 2528a32d)
    [19] = { -538.72, -0.35, -200.45, -0.238 }, -- Eastern Thanalan, ChocoboTaxi 57 (1fac2416, 2528a32d)
}

local registering = false

local function stand_id()
    return EVENT_ID & 0xFFFF
end

function onTalk(target, player)
    local stand = stand_id()
    registering = not player:has_chocobo_taxi_stand(stand)
    if registering then
        player:toggle_chocobo_taxi_stand(stand)
        local also = ALSO_REGISTERS[stand]
        if also ~= nil and not player:has_chocobo_taxi_stand(also) then
            player:toggle_chocobo_taxi_stand(also)
        end
        player:play_scene(SCENE_00000, HIDE_HOTBAR, {1})
    else
        player:play_scene(SCENE_00000, HIDE_HOTBAR, {0})
    end
end

function onReturn(scene, results, player)
    local taxi = results[1] or 0
    player:finish_event()
    if registering or taxi == 0 then
        return
    end

    local start = ROUTE_STARTS[stand_id()]
    if start ~= nil then
        player:ride_chocobo_taxi(taxi, start[1], start[2], start[3], start[4])
    else
        local position = player.position
        player:ride_chocobo_taxi(taxi, position.x, position.y, position.z, player.rotation)
    end
end
