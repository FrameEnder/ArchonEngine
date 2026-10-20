-- Quest: A Royal Reception (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 552):
-- Sequence 0:   ACTOR0 (Momodi) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Bartholomew, the Steps of Thal) talks (scenes 2 and 3), then the cutscene (scene 4,
--               0xF8402EFB), after which the player is at POPRANGE0 in the Hourglass (territory 178, as retail moved
--               them) and the quest at sequence 255. The client offers ACTOR1 while UI8AL and BitFlag8 1 are unset.
--               ACTOR2 (Serendipity) is offered only while the player has RITEM0: scene 5 (only talk; which of the
--               client's scenes 5 and 6 is hers is a guess, retail didn't play them.)
-- Sequence 255: ACTOR3 (Raubahn) rewards it (scene 7, which only returns, then scene 8, a yield: QuestReward, 1; gil
--               only.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BARTHOLOMEW = 2
SCENE_BARTHOLOMEW_2 = 3
SCENE_CUTSCENE = 4
SCENE_SERENDIPITY = 5
SCENE_RAUBAHN = 7
SCENE_FINISH = 8

HOURGLASS = 178 -- the inn, where retail's cutscene leaves the player (Bob's capture of 2026-10-18)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

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
        play(player, SCENE_BARTHOLOMEW, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_SERENDIPITY, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_RAUBAHN, HIDE_HOTBAR)
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
    elseif scene == SCENE_BARTHOLOMEW then
        play(player, SCENE_BARTHOLOMEW_2, HIDE_HOTBAR)
        return
    elseif scene == SCENE_BARTHOLOMEW_2 then
        play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_CUTSCENE then
        player:quest_sequence(EVENT_ID, 255)
        player:finish_event()
        player:change_territory_pop_range(HOURGLASS, POPRANGE0)
        return
    elseif scene == SCENE_RAUBAHN then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
        return
    elseif scene == SCENE_FINISH then
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
