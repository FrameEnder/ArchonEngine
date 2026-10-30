-- The GATE Keepers: they tell of the GATE on now, or the next one, and can send you to its entrance.
--
-- As retail played them (Bob's Chronofoil captures deaf7e28 and 2c98f9e8 of 2026-10-29, and his archive), all with
-- flags 0x2000:
-- Scene 0: a GATE is taking registrations: { type, location, GFATE row }.
-- Scene 1: a GATE is under way that can't be joined any more (Any Way the Wind Blows after its registration, 61a9a4f3):
--          the same parameters.
-- Scene 2: none is on: the next one, { the minute it begins (18:00:00 for 18:00:01's), type, location, row }.
-- Each returns { 1 to be sent to the GATE's entrance, the row }; retail then finished the event and moved the player to
-- the entrance in the Gold Saucer (PrepareZoning with warp type 2, then ActorSetPos 4.5 yalms in front of the
-- entrance's NPC, facing it: the GFATE row's "pop range" is that NPC's layout ID.)
--
-- GFateType: 1 Cliffhanger, 5 Any Way the Wind Blows, 6 Leap of Faith, 7 Air Force One, 8 The Slice Is Right (2-4 are
-- GATEs that are gone.) Locations: 1 Wonder Square East, 2 Event Square, 3 Round Square.

SCENE_REGISTRATION = 0
SCENE_UNDER_WAY = 1
SCENE_NEXT = 2

-- A fade, as retail's warp to the entrance.
WARP_TYPE_FADE = 2

-- The GATE this talk is about, for the warp.
gate = nil

function onTalk(target, player)
    local info = player:gate_info()
    if info.current ~= nil then
        gate = info.current
        if gate.registering then
            player:play_scene(SCENE_REGISTRATION, HIDE_HOTBAR, { gate.kind, gate.location, gate.row })
        else
            player:play_scene(SCENE_UNDER_WAY, HIDE_HOTBAR, { gate.kind, gate.location, gate.row })
        end
    elseif info.next ~= nil then
        gate = info.next
        player:play_scene(SCENE_NEXT, HIDE_HOTBAR, { gate.time, gate.kind, gate.location, gate.row })
    else
        gate = nil
        player:finish_event()
    end
end

function onReturn(scene, results, player)
    player:finish_event()
    if gate ~= nil and results[1] == 1 then
        player:move_to_pop_range(gate.entrance, true, WARP_TYPE_FADE)
    end
    gate = nil
end
