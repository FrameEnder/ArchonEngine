-- Quest: Brotherly Love
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest and its battle: its fixture is tools/questkit/fixtures/724.json):
-- Sequence 0:   ACTOR0 (Komuxio) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene
--               (scene 90, 0xF8402EFB, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Claxio) talks, walks off and fades (scene 2, 0x6001): notice 0, sequence 2.
-- Sequence 2:   ACTOR0 talks (scene 3): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 (Claxio again, elsewhere) asks whether to enter QUESTBATTLE0 (scene 4: YesNoQuestBattle, a
--               yield, 1 for yes), which takes the player to TERRITORYTYPE0 (303). EOBJECT0-2 are shown but can't be
--               targeted (the client's IsTargetingPossible); their scenes (5-10) are empty. Winning the battle sets
--               sequence 255: that's the battle's director (content/questbattle/57.lua), not this script.
-- Sequence 255: ACTOR0 rewards it (scene 11, a cutscene and a yield: QuestReward, 1; gil only.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 90
SCENE_CLAXIO = 2
SCENE_KOMUXIO = 3
SCENE_ENTER_BATTLE = 4
SCENE_REWARD = 11

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
CLAXIO_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001 on retail (he walks off and fades)

NOTICE_CLAXIO = 0
NOTICE_KOMUXIO = 1

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
        play(player, SCENE_CLAXIO, CLAXIO_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_KOMUXIO, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_CLAXIO then
        player:event_notice(EVENT_ID, NOTICE_CLAXIO, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_KOMUXIO then
        player:event_notice(EVENT_ID, NOTICE_KOMUXIO, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
