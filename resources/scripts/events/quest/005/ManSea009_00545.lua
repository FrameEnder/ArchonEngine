-- Quest: A Mizzenmast Repast (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21). Ul'dah's A Royal Reception (ManWil009_00552.lua) with Limsa's actors:
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts), then scene 1 (empty): sequence 1.
-- Sequence 1:   ACTOR1 (Zanthael, in the Lower Decks; the client offers him only while you hold RITEM0, unequipped
--               counts, and UI8AL and BitFlag8 1 are unset) talks (scenes 2 and 3), then the cutscene (scene 4,
--               SET_BASE | HIDE_HOTBAR): you're moved to POPRANGE0 in the Mizzenmast inn (territory 177, as retail moved
--               you) at sequence 255. ACTOR2 is offered while you hold RITEM0: a line (scene 6.)
-- Sequence 255: ACTOR1 takes you to the Command room (scene 8, a yes/no: 1 moves you to POPRANGE1 in TERRITORYTYPE0.)
--               ACTOR3 rewards it (scene 7, a yield with the reward picked.)
-- Scene 5 (another line of ACTOR1's) wasn't played on retail and this script doesn't play it.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR1_2 = 3
SCENE_CUTSCENE = 4
SCENE_ACTOR2 = 6
SCENE_FINISH = 7
SCENE_TO_COMMAND_ROOM = 8

MIZZENMAST = 177 -- the inn, where retail's cutscene leaves you (Bob's Chronofoil capture of 2026-10-21)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played it

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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TO_COMMAND_ROOM, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1 then
        play(player, SCENE_ACTOR1_2, HIDE_HOTBAR)
        return
    elseif scene == SCENE_ACTOR1_2 then
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_CUTSCENE then
        player:quest_sequence(EVENT_ID, 255)
        player:finish_event()
        player:change_territory_pop_range(MIZZENMAST, POPRANGE0)
        return
    elseif scene == SCENE_TO_COMMAND_ROOM then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE1)
            return
        end
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
