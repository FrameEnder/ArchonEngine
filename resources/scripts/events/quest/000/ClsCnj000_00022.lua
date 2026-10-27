-- Quest: Way of the Conjurer (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b23b91ef (2026-10-26,
-- the whole quest; its fixture is tools/questkit/fixtures/22.json).
-- Sequence 0:   ACTOR0 (Madelle) offers the quest (scene 0, a yield returning 1 when accepted, 0 when declined.)
-- Sequence 1:   ACTOR1 (E-Sumi-Yan)'s cutscene (scene 1, retail's flags 0xF8482EFB) ends on a yes/no: 1 moves to
--               sequence 2 with notice 0, 0 leaves it at 1 (the client's script returns 0 for "no".)
-- Sequence 2:   kill three each of ENEMY0 (ground squirrels), ENEMY1 (little ladybugs) and ENEMY2 (forest funguar),
--               counted in UI8AL, UI8BH and UI8BL (the client's IsAcceptEvent stops counting a kind at 3.) Each kill
--               sends notice 1, 2 or 3 (the kind's todo) with [count, 3]; when a kind reaches 3 UI8AH counts it as
--               done, and when all three have, sequence 255 with every variable cleared (retail: 9 kills, in any
--               order.)
-- Sequence 255: ACTOR1's reward (scene 2, a yield returning 1.) The Quest sheet's SystemReward (unlock 21, the
--               Hunting Log) is given by finish_quest.

SCENE_OFFER = 0
SCENE_CUTSCENE = 1
SCENE_REWARD = 2

-- Retail's flags for the cutscene (Bob's Chronofoil capture b23b91ef.)
CUTSCENE_FLAGS = 0xF8482EFB

-- Each enemy: its todo (and notice), and where its count is kept (byte, high nibble.)
ENEMIES = {
    [ENEMY0] = { notice = 1, byte = 0, high = false }, -- UI8AL
    [ENEMY1] = { notice = 2, byte = 1, high = true }, -- UI8BH
    [ENEMY2] = { notice = 3, byte = 1, high = false }, -- UI8BL
}
KILLS_NEEDED = 3
KINDS = 3

NOTICE_TALKED = 0

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
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
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    local enemy = ENEMIES[base_id]
    if enemy == nil or player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    local count = getVar(player, enemy.byte, enemy.high)
    if count >= KILLS_NEEDED then
        return
    end
    count = count + 1
    player:event_notice(EVENT_ID, enemy.notice, { count, KILLS_NEEDED })
    if count < KILLS_NEEDED then
        setVar(player, enemy.byte, enemy.high, count)
        return
    end
    local done = getVar(player, 0, true) + 1 -- UI8AH
    if done >= KINDS then
        player:quest_sequence(EVENT_ID, 255)
        player:set_quest_var(EVENT_ID, 0, 0)
        player:set_quest_var(EVENT_ID, 1, 0)
        return
    end
    setVar(player, enemy.byte, enemy.high, count)
    setVar(player, 0, true, done)
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
    elseif scene == SCENE_CUTSCENE then
        if results[1] == 1 then
            player:event_notice(EVENT_ID, NOTICE_TALKED, {})
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
