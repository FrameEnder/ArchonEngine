-- Quest travel: an NPC who offers to take the player somewhere ("go there?", a talk and a yes/no) does so in every
-- sequence the client offers them, not only the one a recording caught. Loaded by the server after a quest's own
-- script when `resources/data/quest_travel.json` has entries for it (`questkit travel` writes them; see
-- docs/src/fork/quests.md, Quest travel). It answers those talks and leaves everything else to the script.
--
-- QUEST_TRAVEL (set by the server): a list of { actor = ENpc base, scenes = { ["<sequence>"] = scene }, flags,
-- move = { call, args... } }, sequence 0 being before the quest is accepted. On yes the player is moved as the
-- quest's recorded ride moved them (the same call and numbers the script made on retail's step).
--
-- Every Little Thing She Does Is Magitek (1005): Slafborn takes the player into Cid's workshop in sequence 3 (the
-- recording), and also before it's accepted (scene 2), in sequence 1 (4) and 255 (51), as the client offers; a player
-- who left before accepting it couldn't get back in (Bob's bundle 20261114-024122).

local script_onTalk = onTalk
local script_onYield = onYield
local script_onReturn = onReturn

-- The ride being asked about: { scene, move }.
local travel_ride = nil

local function ride_for(player)
    local sequence = player:get_quest_sequence(EVENT_ID) or 0
    for _, entry in ipairs(QUEST_TRAVEL or {}) do
        if entry.actor == BASE_ID then
            local scene = entry.scenes[tostring(sequence)]
            if scene ~= nil then
                return { scene = scene, flags = entry.flags, move = entry.move }
            end
        end
    end
    return nil
end

local function travel(player, move)
    local call = move[1]
    if call == "change_territory" then
        player:change_territory(move[2], move[3], move[4])
    elseif call == "change_territory_pop_range" then
        player:change_territory_pop_range(move[2], move[3])
    elseif call == "move_to_pop_range" then
        player:move_to_pop_range(move[2], move[3])
    end
end

-- The answer to the ride's question: yes moves the player, anything else ends the talk. True if it was the ride's.
local function answered(scene, results, player)
    local ride = travel_ride
    if ride == nil or scene ~= ride.scene then
        return false
    end
    travel_ride = nil
    player:finish_event()
    if results ~= nil and results[1] == 1 then
        travel(player, ride.move)
    end
    return true
end

function onTalk(target, player)
    local ride = ride_for(player)
    if ride ~= nil then
        travel_ride = ride
        player:play_scene(ride.scene, ride.flags, {})
        return
    end
    travel_ride = nil
    if script_onTalk ~= nil then
        script_onTalk(target, player)
    else
        player:finish_event()
    end
end

function onYield(scene, id, results, player)
    if answered(scene, results, player) then
        return
    end
    if script_onYield ~= nil then
        script_onYield(scene, id, results, player)
    end
end

function onReturn(scene, results, player)
    if answered(scene, results, player) then
        return
    end
    if script_onReturn ~= nil then
        script_onReturn(scene, results, player)
    end
end
