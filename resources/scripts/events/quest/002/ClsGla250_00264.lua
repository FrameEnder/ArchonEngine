-- Quest: On Holy Ground (the Gladiators' level 25 quest)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil capture
-- accc33a9 (2026-11-06, the whole quest on a Gladiator, its quest battle won): its fixture is
-- tools/questkit/fixtures/264.json.
-- Sequence 0:   ACTOR0 (Mylla, in Ul'dah - Steps of Thal) offers it (scene 0, a yield: the client accepts it there,
--               returning 1): sequence 1.
-- Sequence 1:   ACTOR1 (the veteran gladiator, in Western Thanalan; offered while BitFlag8 1 is unset, scene 1, with
--               its pop-up message): as it ends ENEMY0-2 (three of BNpcBase 1243) come out for the player (layer
--               QB_ClsGla250_001 of the wil_w1f1 drop-in, retail's positions, level 25 and 445 HP), 0.1 s later on
--               retail. Each one down counts in UI8AL; the third: notice 0, sequence 2 with every variable 0.
-- Sequence 2:   ACTOR1 again (scene 2): notice 1, sequence 3.
-- Sequence 3:   ACTOR0 (scene 3): notice 2, sequence 4. ACTOR1 has a line meanwhile (scene 4.)
-- Sequence 4:   ACTOR2 (Lulutsu, in Ul'dah - Steps of Thal, scene 5): notice 3, sequence 5.
-- Sequence 5:   ACTOR3 (Aldis, in Eastern Thanalan, scene 6): YesNoQuestBattle; 1 enters QUESTBATTLE0 (22) in
--               TERRITORYTYPE0 (256), and it can be tried again after a loss. Winning it moves to sequence 255
--               (content/questbattle/22.lua.) EOBJECT0's scenes (7, 8) are empty.
-- Sequence 255: ACTOR0 rewards it (scene 9, a yield: QuestReward, 1 and the optional item picked; closing the window
--               instead leaves UI8A 0x11, as retail does: docs/src/fork/quests.md.)

SCENE_OFFER = 0
SCENE_GLADIATOR = 1
SCENE_GLADIATOR_AFTER = 2
SCENE_MYLLA = 3
SCENE_LULUTSU = 5
SCENE_BATTLE = 6
SCENE_REWARD = 9

-- The three to bring down in sequence 1, counted in UI8AL.
ENEMIES = { ENEMY0, ENEMY1, ENEMY2 }
ENEMIES_TO_BEAT = 3
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- The other lines, by sequence: the client's scene for each NPC it offers then.
LINES = {
    [3] = { [ACTOR1] = 4 },
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
        play(player, SCENE_MYLLA, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_LULUTSU, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR3 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
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
                player:event_notice(EVENT_ID, 0, {})
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
    elseif scene == SCENE_LULUTSU then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
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
