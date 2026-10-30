-- Quest: Nouveau Riche
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest and its battle: its fixture is tools/questkit/fixtures/3862.json):
-- Sequence 0:   ACTOR0 (Buscarron) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Nouveau Riche (740), is accepted), then the cutscene (scene 1,
--               0xF8406EFB, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Laurentius) talks, walks off and fades (scene 2, 0x2001): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (Laurentius, further on) talks (scene 3); as it ends ENEMY0 (a layout ID) comes out for the
--               player, as retail's did 0.4 s after. Defeating it: notice 1, sequence 3.
-- Sequence 3:   ACTOR2 talks, walks off and fades (scene 6, 0x2001): notice 2, sequence 4.
-- Sequence 4:   ACTOR3 (Laurentius, at the end) asks whether to enter QUESTBATTLE0 (scene 8: YesNoQuestBattle, a yield,
--               1 for yes), which takes the player to TERRITORYTYPE0 (232). Winning sets sequence 255: that's the
--               battle's director (content/questbattle/13.lua), not this script.
-- Sequence 255: ACTOR0 rewards it (scene 11, a yield: QuestReward, 1; gil only.) Retail then completed the old Nouveau
--               Riche (740) too: FinishQuest 3862, then 740.
-- The enemy is layer QB_XxaUsa308_001 of the South Shroud drop-in, at retail's position, level and HP. EOBJECT0 and
-- EOBJECT1's scenes (5, 7, 9, 10) are empty, scene 4 a log message about the enemy: not played.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_LAURENTIUS = 2
SCENE_LAURENTIUS_AMBUSH = 3
SCENE_LAURENTIUS_AFTER = 6
SCENE_ENTER_BATTLE = 8
SCENE_REWARD = 11

-- Retail's flags for the opening cutscene (Bob's Chronofoil capture caa5e908): SET_BASE | HIDE_HOTBAR with 0x4000 more
-- (INVINCIBLE.)
ACCEPTED_FLAGS = SET_BASE | HIDE_HOTBAR | INVINCIBLE -- 0xF8406EFB
WALK_OFF_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

NOTICE_LAURENTIUS = 0
NOTICE_ENEMY = 1
NOTICE_LAURENTIUS_AFTER = 2
-- The quest this one replaced, completed with it (retail's FinishQuest 740 right after 3862's.)
QUEST_ALSO_COMPLETED = 740

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
        play(player, SCENE_LAURENTIUS, WALK_OFF_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_LAURENTIUS_AMBUSH, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_LAURENTIUS_AFTER, WALK_OFF_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR3 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if layout_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_ENEMY, {})
    player:quest_sequence(EVENT_ID, 3)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, ACCEPTED_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_LAURENTIUS then
        player:event_notice(EVENT_ID, NOTICE_LAURENTIUS, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_LAURENTIUS_AMBUSH then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_LAURENTIUS_AFTER then
        player:event_notice(EVENT_ID, NOTICE_LAURENTIUS_AFTER, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
