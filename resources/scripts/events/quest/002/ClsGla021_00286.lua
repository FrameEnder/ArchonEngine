-- Quest: My First Gladius
--
-- Written from the client's quest script (kawari-questdump, 2026-11-04) and checked against Bob's Chronofoil capture
-- d3379099 (2026-11-04, on a Conjurer who had just joined: its fixture is tools/questkit/fixtures/286.json). The other
-- classes' first Gladiator hunt, after Way of the Gladiator (285); Way of the Gladiator for Gladiators (253) is the
-- same hunt after its cutscene.
-- Sequence 0:   ACTOR0 (the guildmaster) offers it (scene 0, a yield: QuestOffer, 1 when accepted): sequence 1.
-- Sequence 1:   three each of ENEMY0, ENEMY1 and ENEMY2 (BNpcBases, the zones' own), in any order: UI8AL, UI8BH and
--               UI8BL count them, notice 0, 1 or 2 with [count, 3] at each; UI8AH counts the kinds done (retail's 1
--               and 2 after the first and second). The last: the variables cleared, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 1, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_REWARD = 1

KILLS_NEEDED = 3
HUNTING_SEQUENCE = 1

-- Each enemy's counter: the variable byte, whether it's the high nibble, its notice.
COUNTERS = {
    { enemy = "ENEMY0", byte = 0, high = false, notice = 0 }, -- UI8AL
    { enemy = "ENEMY1", byte = 1, high = true, notice = 1 },  -- UI8BH
    { enemy = "ENEMY2", byte = 1, high = false, notice = 2 }, -- UI8BL
}
-- UI8AH: how many kinds are done.
KINDS_DONE = { byte = 0, high = true }

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getNibble(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return (value >> 4) & 0x0F
    end
    return value & 0x0F
end

function setNibble(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= HUNTING_SEQUENCE then
        return
    end
    for _, counter in ipairs(COUNTERS) do
        if base_id == _G[counter.enemy] then
            local count = getNibble(player, counter.byte, counter.high)
            if count >= KILLS_NEEDED then
                return
            end
            count = count + 1
            setNibble(player, counter.byte, counter.high, count)
            player:event_notice(EVENT_ID, counter.notice, { count, KILLS_NEEDED })
            if count == KILLS_NEEDED then
                local done = getNibble(player, KINDS_DONE.byte, KINDS_DONE.high) + 1
                if done == #COUNTERS then
                    for byte = 0, 5 do
                        player:set_quest_var(EVENT_ID, byte, 0)
                    end
                    player:quest_sequence(EVENT_ID, 255)
                else
                    setNibble(player, KINDS_DONE.byte, KINDS_DONE.high, done)
                end
            end
            return
        end
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, HUNTING_SEQUENCE)
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
        end
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
