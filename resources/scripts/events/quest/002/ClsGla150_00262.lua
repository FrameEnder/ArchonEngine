-- Quest: That Old Familiar Feeling (the Gladiators' level 15 quest)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil capture
-- 941895d2 (2026-11-06, the whole quest on a Gladiator, its quest battle won): its fixture is
-- tools/questkit/fixtures/262.json.
-- Sequence 0:   ACTOR0 (Mylla, in Ul'dah - Steps of Thal) offers it (scene 0, a yield: the client accepts it there,
--               returning 1): sequence 1.
-- Sequence 1:   ACTOR1 (the injured gladiator, in Eastern Thanalan, scene 1, with its pop-up message): as it ends
--               ENEMY0 and ENEMY1 (two Qiqirn traffickers, BNpcBase 1244) come out for the player (layer
--               QB_ClsGla150_001 of the wil_w1f3 drop-in, retail's positions, level 15 and 319 HP), 0.4 s later on
--               retail. Each one down counts in UI8AL; the second: notice 0 ([1, 1] on retail), sequence 2 with every
--               variable 0.
-- Sequence 2:   ACTOR1 again (scene 15): notice 1, sequence 3.
-- Sequence 3:   ACTOR0 (scene 25, a cutscene: retail's 0xF8402EFB): notice 2, sequence 4.
-- Sequence 4:   ACTOR3 (Aldis, in Ul'dah - Steps of Nald, scene 40): YesNoQuestBattle; 1 enters QUESTBATTLE0 (20) in
--               TERRITORYTYPE0 (254), and it can be tried again after a loss. Winning it moves to sequence 255
--               (content/questbattle/20.lua.)
-- Sequence 255: ACTOR0 rewards it (scene 44, a cutscene and a yield: QuestReward, 1 and the optional item picked;
--               closing the window instead leaves UI8A 0x11, as retail does: docs/src/fork/quests.md.)
-- The client's scenes follow its IsAcceptEvent order (two each for the enemies and objects); the ones with a line in
-- them are LINES below (the nervous merchant, the gladiator and Mylla in the other sequences.) The objects' scenes are
-- empty.

SCENE_OFFER = 0
SCENE_GLADIATOR = 1
SCENE_GLADIATOR_AFTER = 15
SCENE_MYLLA = 25
SCENE_BATTLE = 40
SCENE_REWARD = 44

-- Retail's flags (Bob's Chronofoil capture 941895d2.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB

-- The two to bring down in sequence 1, counted in UI8AL.
ENEMIES = { ENEMY0, ENEMY1 }
ENEMIES_TO_BEAT = 2
-- Notice 0's arguments, as retail sent them (Bob's Chronofoil capture 941895d2.)
NOTICE_BEATEN_ARGS = { 1, 1 }
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- The other lines, by sequence: the client's scene for each NPC it offers then.
LINES = {
    [1] = { [ACTOR2] = 14 },
    [3] = { [ACTOR2] = 26, [ACTOR1] = 27 },
    [4] = { [ACTOR0] = 41 },
}

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
        play(player, SCENE_GLADIATOR, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_GLADIATOR_AFTER, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_MYLLA, CUTSCENE_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR3 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    for _, enemy in ipairs(ENEMIES) do
        if layout_id == enemy then
            local beaten = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
            if beaten >= ENEMIES_TO_BEAT then
                -- The count cleared with the new sequence (retail's UpdateQuest: sequence 2, every variable 0.)
                player:event_notice(EVENT_ID, 0, NOTICE_BEATEN_ARGS)
                player:set_quest_var(EVENT_ID, 0, 0)
                player:quest_sequence(EVENT_ID, 2)
            else
                player:set_quest_var(EVENT_ID, 0, beaten)
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
    elseif scene == SCENE_GLADIATOR then
        player:finish_event()
        -- Ones still out aren't brought out again.
        for _, enemy in ipairs(ENEMIES) do
            player:spawn_quest_bnpc(enemy)
        end
        return
    elseif scene == SCENE_GLADIATOR_AFTER then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_MYLLA then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
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
