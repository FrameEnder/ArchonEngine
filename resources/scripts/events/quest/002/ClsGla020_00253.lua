-- Quest: Way of the Gladiator
--
-- Written from the client's quest script (kawari-questdump, 2026-11-04) and checked against Bob's Chronofoil capture
-- 208c3716 (2026-11-03, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/253.json):
-- Sequence 0:   ACTOR0 (at the Gladiators' Guild's door) offers it (scene 0, a yield: QuestOffer, 1 when accepted):
--               sequence 1.
-- Sequence 1:   ACTOR1 (the guildmaster): her cutscene (scene 1, retail's flags 0xF8482EFB, a yield: its YesNo, 1 for
--               yes; no cancels it and nothing happens): notice 0, sequence 2.
-- Sequence 2:   three each of ENEMY0, ENEMY1 and ENEMY2 (BNpcBases, the zones' own), in any order: UI8AL, UI8BH and
--               UI8BL count them (the client's IsTodoChecked), notice 1, 2 or 3 with [count, 3] at each; UI8AH counts
--               the kinds done (retail's 1 and 2 after the first and second). The last: the variables cleared,
--               sequence 255.
-- Sequence 255: ACTOR1 rewards it (scene 5, a yield: QuestReward, 1.) The Quest sheet's SystemReward (unlock link 21,
--               retail's UnlockLink at the reward) is the server's finish_quest.

SCENE_OFFER = 0
SCENE_GUILDMASTER = 1
SCENE_REWARD = 5

CUTSCENE_FLAGS = 0xF8482EFB -- retail's (Bob's Chronofoil capture 208c3716)

NOTICE_GUILDMASTER = 0
KILLS_NEEDED = 3

-- Each enemy's counter: the variable byte, whether it's the high nibble, its notice.
COUNTERS = {
    { enemy = "ENEMY0", byte = 0, high = false, notice = 1 }, -- UI8AL
    { enemy = "ENEMY1", byte = 1, high = true, notice = 2 },  -- UI8BH
    { enemy = "ENEMY2", byte = 1, high = false, notice = 3 }, -- UI8BL
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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_GUILDMASTER, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
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
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_GUILDMASTER then
        if results[1] == 1 then
            player:event_notice(EVENT_ID, NOTICE_GUILDMASTER, {})
            player:quest_sequence(EVENT_ID, 2)
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
