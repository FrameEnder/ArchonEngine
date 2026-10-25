-- Quest: A Wild Rose by Any Other Name
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-25), Sapphire's script and retail (Bob's
-- Chronofoil capture of 2026-10-25, questkit replay 510):
-- Sequence 0:   ACTOR0 (Minfilia, the Waking Sands) offers it (scene 0, a yield: a menu, then QuestOffer; 1 when
--               accepted), then the cutscene (scene 50, CUTSCENE_FLAGS, which ends with QuestAccepted): sequence 1.
--               Retail then moved the player within the Waking Sands (PrepareZoning to the same territory, then
--               ActorSetPos to WAKING_SANDS_AFTER_CUTSCENE, warp type 18); the quest has no pop range for it.
-- Sequence 1:   ACTOR2 (Thancred, beside you after the cutscene) talks and walks off (scene 2, HIDE_HOTBAR |
--               NO_DEFAULT_CAMERA | INVINCIBLE as retail): notice 0 with [1, 1], sequence 255. The client offers
--               ACTOR1 (Thancred elsewhere) too while UI8AL and BitFlag8 1 are unset; his talk (scene 1, Sapphire's
--               choice) does the same here. Retail went straight to 255 with no variables set.
-- Sequence 255: ACTOR3 (Isembard, Ul'dah) rewards it (scene 3, a yield: QuestReward, 1 and the chosen item.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 50
SCENE_THANCRED_ELSEWHERE = 1
SCENE_THANCRED = 2
SCENE_REWARD = 3

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played it (Bob's Chronofoil capture of 2026-10-25)
THANCRED_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001, as retail played scene 2

-- Where retail put the player after the cutscene (ActorSetPos in Bob's Chronofoil capture of 2026-10-25.)
WAKING_SANDS = 212
WAKING_SANDS_AFTER_CUTSCENE = { x = 35.986244, y = 1.2000016, z = 0.8130567 }
WAKING_SANDS_AFTER_CUTSCENE_ROTATION = 1.5721146

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
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_THANCRED, THANCRED_FLAGS)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_THANCRED_ELSEWHERE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
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
        player:finish_event()
        player:change_territory(WAKING_SANDS, WAKING_SANDS_AFTER_CUTSCENE, WAKING_SANDS_AFTER_CUTSCENE_ROTATION)
        player:quest_sequence(EVENT_ID, 1)
        return
    elseif scene == SCENE_THANCRED or scene == SCENE_THANCRED_ELSEWHERE then
        player:event_notice(EVENT_ID, 0, { 1, 1 })
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
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
