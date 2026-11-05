-- Quest: Ul'dah's Most Wanted (the Gladiators' level 10 quest)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil capture
-- a763f0a0 (2026-11-05, the whole quest on a Gladiator, its quest battle won): its fixture is
-- tools/questkit/fixtures/261.json.
-- Sequence 0:   ACTOR0 (Mylla, in Ul'dah - Steps of Nald) offers it (scene 0, a yield: the client accepts it there,
--               returning 1): sequence 1.
-- Sequence 1:   ACTOR1 (the gutless gladiator, in Central Thanalan, scene 1): as it ends ENEMY0 and ENEMY1 come out
--               for the player (layer QB_ClsGla100_001 of the wil_w1 drop-in, retail's positions, level and HP),
--               0.4 s later on retail. Each one down counts in UI8AL (the client's todo, 2); the second: notice 0,
--               sequence 2.
-- Sequence 2:   ACTOR1 again (scene 2, he walks off): notice 1, sequence 3.
-- Sequence 3:   ACTOR0 (scene 3, a cutscene: retail's 0xF8402EFB): notice 2, sequence 4.
-- Sequence 4:   EOBJECT0 (the destination, in Central Thanalan): its empty scene 4, EVENT_ACTION_WAITING's bar (2 s
--               on retail), then scene 5: YesNoQuestBattle; 1 enters QUESTBATTLE0 (19) in TERRITORYTYPE0 (253), and it
--               can be tried again after a loss. Winning it moves to sequence 5 (content/questbattle/19.lua.) ACTOR2
--               (Bruce the Big) has a line meanwhile (scene 7).
-- Sequence 5:   ACTOR2 (scene 8, he walks off): notice 4, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 9, a yield: QuestReward, 1 and the optional item picked.)
-- Scene 6 is empty and nothing plays it.

SCENE_OFFER = 0
SCENE_GLADIATOR = 1
SCENE_GLADIATOR_LEAVES = 2
SCENE_MYLLA = 3
SCENE_DESTINATION = 4
SCENE_BATTLE = 5
SCENE_BRUCE = 7
SCENE_BRUCE_LEAVES = 8
SCENE_REWARD = 9

-- Retail's flags (Bob's Chronofoil capture a763f0a0.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
EVENT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

-- The two to bring down in sequence 1, counted in UI8AL.
ENEMIES = { ENEMY0, ENEMY1 }
ENEMIES_TO_BEAT = 2

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
        play(player, SCENE_GLADIATOR_LEAVES, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_MYLLA, CUTSCENE_FLAGS)
    elseif sequence == 4 and BASE_ID == EOBJECT0 then
        play(player, SCENE_DESTINATION, EVENT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_BRUCE, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR2 then
        play(player, SCENE_BRUCE_LEAVES, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The waiting bar at the destination finished: the battle's question.
function onEventActionCast(player)
    if current_scene ~= nil or player:get_quest_sequence(EVENT_ID) ~= 4 then
        player:finish_event()
        return
    end
    play(player, SCENE_BATTLE, EVENT_FLAGS)
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
    elseif scene == SCENE_GLADIATOR_LEAVES then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_MYLLA then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_DESTINATION then
        -- The bar, then the battle's question (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING)
        return
    elseif scene == SCENE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_BRUCE_LEAVES then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
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
