-- Quest: The Rose and the Unicorn (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/890.json and `890-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   ACTOR1 asks (scene 2, the client's own menu: a wrong answer cancels the scene there), the cutscene
--               (scene 3, 0xF8402EFB), then the empty scene 4: notice 0 [ITEM0's icon], sequence 255 with ITEM0
--               (UI8BH = 1.)
-- Sequence 255: ACTOR2 (scene 5), then the cutscene and reward (scene 6, 0xF8402EFB, a yield: QuestReward, 1 and the
--               optional item picked; closing the window leaves UI8A 0x11.) Retail completed Feats of Strength (896)
--               with it, without rewards (FinishQuest 890 then 896, both characters.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_QUESTION = 2
SCENE_CUTSCENE = 3
SCENE_AFTER = 4
SCENE_ACTOR2 = 5
SCENE_REWARD = 6

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
-- ITEM0's icon, on the notice (Bob's Chronofoil captures of 2026-11-06.)
ITEM0_ICON = 25918
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did.
QUEST_ALSO_COMPLETED = 896

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
        play(player, SCENE_QUESTION, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_QUESTION then
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_CUTSCENE then
        play(player, SCENE_AFTER, HIDE_HOTBAR)
        return
    elseif scene == SCENE_AFTER then
        player:event_notice(EVENT_ID, 0, { ITEM0_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_ACTOR2 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
            return
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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
