-- Quest: The Scions of the Seventh Dawn
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-25), Sapphire's script and retail (Bob's
-- Chronofoil capture of 2026-10-25, questkit replay 509):
-- Sequence 0:   ACTOR0 (Momodi, Ul'dah) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then talks
--               (scene 1, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   ACTOR1 (Tataru, in Vesper Bay) and the cutscene (scene 2, CUTSCENE_FLAGS): notice 0, sequence 255.
-- Sequence 255: ACTOR2 (in the Waking Sands) rewards it (scene 3, a yield: QuestReward, 1 and the chosen item), then the
--               cutscenes (scene 4, CUTSCENE_FLAGS, given the chosen item and returning it; it also shows the
--               retainer unlock image and how-to.) Retail set UI8AH and UI8AL to 1 as scene 4 started (the client's
--               todo reads UI8AL), and after it moved the player within the Waking Sands to POPRANGE0 (PrepareZoning
--               to the same territory, then FinishQuest.) ACTOR1 has a line here (scene 5, Sapphire's choice too.)
-- The retainer unlock (UnlockLink 3) is the Quest sheet's: the server's finish_quest sends it.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TATARU = 2
SCENE_REWARD = 3
SCENE_FINISH = 4
SCENE_TATARU_AFTER = 5

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played them (Bob's Chronofoil capture of 2026-10-25)

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The reward picked in scene 3, which scene 4 is given and finish_quest gets.
chosen_item = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TATARU, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TATARU_AFTER, HIDE_HOTBAR)
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
    elseif scene == SCENE_TATARU then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            chosen_item = results[2] or 0
            play(player, SCENE_FINISH, CUTSCENE_FLAGS, { chosen_item })
            player:set_quest_var(EVENT_ID, 0, 0x11)
            return
        end
    elseif scene == SCENE_FINISH then
        player:finish_event()
        player:move_to_pop_range(POPRANGE0, true)
        player:finish_quest(EVENT_ID, chosen_item)
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
