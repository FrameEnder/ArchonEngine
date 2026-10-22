-- Quest: Feint and Strike (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21; the quest battle, content/questbattle/36.lua, from the same capture: lost three times,
-- then won):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts), then the cutscene (scene 1, SET_BASE | HIDE_HOTBAR):
--               sequence 1 with UI8BH = 1 (ITEM0).
-- Sequence 1:   hand ITEM0 to ACTOR1 (scene 2, an NpcTrade: 1 if given), then the cutscene (scene 3): notice 0,
--               sequence 2, the variables cleared.
-- Sequence 2:   ACTOR2: the quest battle's yes/no (scene 4, YesNoQuestBattle, a yield: 1 enters QUESTBATTLE0 in
--               TERRITORYTYPE0); after a loss it's the same again. ACTOR3, ACTOR4 and ACTOR5 only have a line each
--               (scenes 5, 6, 7). The client offers EOBJECT0 too, but its scenes (8, 9) are empty. Winning the battle
--               moves the quest to sequence 3.
-- Sequence 3:   back in TERRITORYTYPE1, the cutscene (scene 10): sequence 255.
-- Sequence 255: ACTOR1 (scene 11), then the cutscene and reward (scene 12, a yield with the reward picked.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 2
SCENE_TRADED = 3
SCENE_ENTER_BATTLE = 4
SCENE_AFTERMATH = 10
SCENE_ACTOR1 = 11
SCENE_FINISH = 12

-- The lines in sequence 2, by actor.
LINES = { [ACTOR3] = 5, [ACTOR4] = 6, [ACTOR5] = 7 }

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played the cutscenes

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
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 2 and LINES[BASE_ID] ~= nil then
        play(player, LINES[BASE_ID], HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Back in Western La Noscea after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 3 then
        play(player, SCENE_AFTERMATH, CUTSCENE_FLAGS)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_TRADED then
        player:set_quest_var(EVENT_ID, 1, 0)
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_ACTOR1 then
        play(player, SCENE_FINISH, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_FINISH then
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
