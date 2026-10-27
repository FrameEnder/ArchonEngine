-- Quest: What Poor People Think (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/619.json).
-- Sequence 0:   ACTOR0 (Isembard) offers it (scene 0, a yield returning 1 when accepted; the client accepts it in it):
--               sequence 1 with ITEM0 (UI8BH 1; the client lists it as a key item from that).
-- Sequence 1:   ACTOR1 (Ungust) takes ITEM0 (trade scene 1, a yield: 1 when handed over), then scene 2: notice 0,
--               sequence 2, the variables cleared.
-- Sequence 2:   the three paupers, in any order (the client offers each while UI8AL is below 3 and its BitFlag8 is
--               unset: ACTOR2 1, ACTOR3 2, ACTOR4 3): their scene (3, 4 or 5, with 0x6001; they walk off and fade),
--               then their flag, one more done (UI8AL) and notice 1 [done, 3]. The third clears the variables and goes
--               to sequence 3. ACTOR1 meanwhile: scene 6.
-- Sequence 3:   ACTOR1 (scene 7, 0x6001): notice 2, sequence 255.
-- Sequence 255: ACTOR0's reward (scene 8, a yield: 1 and the item chosen.)

SCENE_OFFER = 0
SCENE_TRADE = 1
SCENE_TRADED = 2
SCENE_UNGUST_WAITING = 6
SCENE_UNGUST_AFTER = 7
SCENE_REWARD = 8

-- Retail's flags (Bob's Chronofoil capture b8432613.)
PAUPER_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001

NOTICE_TRADED = 0
NOTICE_PROGRESS = 1
NOTICE_UNGUST = 2

-- Each pauper: its scene and its BitFlag8.
PAUPERS = {
    [ACTOR2] = { scene = 3, flag = 1 },
    [ACTOR3] = { scene = 4, flag = 2 },
    [ACTOR4] = { scene = 5, flag = 3 },
}
PAUPERS_NEEDED = 3

-- BitFlag8 n is bit 8 - n of UI8F.
function hasFlag(player, flag)
    return player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1)) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

function clearVars(player)
    for byte = 0, 5 do
        player:set_quest_var(EVENT_ID, byte, 0)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local pauper = PAUPERS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and pauper ~= nil and (player:get_quest_var(EVENT_ID, 0) & 0x0F) < PAUPERS_NEEDED
        and not hasFlag(player, pauper.flag) then
        play(player, pauper.scene, PAUPER_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_UNGUST_WAITING, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_UNGUST_AFTER, PAUPER_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- One pauper asked.
function asked(player, flag)
    local done = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1 -- UI8AL
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { done, PAUPERS_NEEDED })
    if done >= PAUPERS_NEEDED then
        clearVars(player)
        player:quest_sequence(EVENT_ID, 3)
        return
    end
    player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | done)
    setFlag(player, flag)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local sequence = player:get_quest_sequence(EVENT_ID)

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        end
    elseif scene == SCENE_TRADE then
        -- 1 when the item was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:event_notice(EVENT_ID, NOTICE_TRADED, {})
        clearVars(player)
        player:quest_sequence(EVENT_ID, 2)
    elseif sequence == 2 and scene >= 3 and scene <= 5 then
        for _, pauper in pairs(PAUPERS) do
            if pauper.scene == scene then
                asked(player, pauper.flag)
            end
        end
    elseif scene == SCENE_UNGUST_AFTER then
        player:event_notice(EVENT_ID, NOTICE_UNGUST, {})
        player:quest_sequence(EVENT_ID, 255)
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
