-- Quest: On to the Drydocks (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts) and hands over ITEM0: accepted straight into sequence
--               255 with UI8BH = 1.
-- Sequence 255: hand ITEM0 to ACTOR1 (scene 1, an NpcTrade: 1 if given), then the cutscene and reward (scene 100,
--               SET_BASE | HIDE_HOTBAR, a yield with the reward picked.)

SCENE_OFFER = 0
SCENE_TRADE = 1
SCENE_FINISH = 100

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB

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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
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
            player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        end
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, CUTSCENE_FLAGS)
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
