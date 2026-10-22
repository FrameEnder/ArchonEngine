-- Quest: Victory in Peril (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21; the quest battle, content/questbattle/24.lua, from the same capture: lost, then won):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts): sequence 1.
-- Sequence 1:   ACTOR1 (scene 1) or ACTOR2 (scene 2; the todo is at either): notice 0 [1, 1], sequence 2.
-- Sequence 2:   ACTOR3 (scene 3, he runs off): notice 1, sequence 3.
-- Sequence 3:   ACTOR0: the quest battle's yes/no (scene 4, YesNoQuestBattle, a yield: 1 enters QUESTBATTLE0 in
--               TERRITORYTYPE0); after a loss it's the same again. Winning it moves the quest to sequence 4.
-- Sequence 4:   ACTOR4 (scene 7): notice 3, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 8, a yield: 1 completes it). Closing the reward window (0) left UI8AH and
--               UI8AL at 1 on retail, so the script does the same; the next talk plays scene 8 again.

SCENE_OFFER = 0
SCENE_ACTOR1 = 1
SCENE_ACTOR2 = 2
SCENE_ACTOR3 = 3
SCENE_ENTER_BATTLE = 4
SCENE_ACTOR4 = 7
SCENE_FINISH = 8

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
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        play(player, SCENE_ACTOR3, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        play(player, SCENE_ACTOR4, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_ACTOR1 or scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 0, { 1, 1 })
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR3 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_ACTOR4 then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
        else
            player:set_quest_var(EVENT_ID, 0, 0x11) -- UI8AH = UI8AL = 1, as retail after the window was closed
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
