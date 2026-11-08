-- Quest: Into the Eye of the Storm (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (18a89a32, 4bcbd425 and de5beba7, each the whole quest; fixtures
-- tools/questkit/fixtures/953.json, 953-18a89a32.json, 953-de5beba7.json):
-- Sequence 0:   ACTOR0 (Cid, in New Gridania) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then his
--               cutscene (scene 1, 0xF8402EFB, CUT_GAIUSC10210), where the client accepts it: sequence 255 at once.
--               ACTOR1 (Lionnellais) asks a yes/no meanwhile (scene 2), unseen on retail: "yes" is taken to take the
--               player to POPRANGE0 in New Gridania (TERRITORYTYPE0), by Cid (a guess, as 952's scene 9.)
-- Sequence 255: ACTOR2 (Lamberteint, in Eastern Thanalan: the player makes their own way there) rewards it (scene 3, a
--               yield: QuestReward, 1.) Retail then completed All Due Precautions (954, a quest no longer offered) too:
--               FinishQuest 953, then 954, with one reward, in all three runs.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_LIONNELLAIS = 2
SCENE_REWARD = 3

-- Retail's flags (Bob's Chronofoil captures of 2026-11-08.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11
-- The quest completed with this one (retail's FinishQuest 954 right after 953's.)
QUEST_ALSO_COMPLETED = 954

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
    elseif sequence == nil and BASE_ID == ACTOR1 then
        play(player, SCENE_LIONNELLAIS, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
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
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_LIONNELLAIS then
        if results[1] == 1 and POPRANGE0 ~= nil then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
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
