-- Quest: Till Sea Swallows All (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-10-28) and the other two
-- companies' retail runs: none of this one has been recorded yet.
-- The three companies' first steps (683 the Twin Adders', 684 the Maelstrom's, 685 the Immortal Flames') are the same
-- script with each company's people and ORDER_OF_*; they enlist the player.
-- Sequence 0:   ACTOR0 (the storm personnel officer, in Limsa Lominsa) offers it (scene 0, a yield returning 1 when
--               accepted), then the cutscene (scene 1, retail's flags 0xF8402EFB), then the enlistment (scene 2):
--               the client's GrandCompanyAssignment yields asking for the company (the Maelstrom, 1), which the
--               server makes the player's (rank 1, the company's achievement, GrandCompanyInfo) and answers with 1;
--               once the scene ends, the quest is accepted, UnlockLink 22 comes (retail's ToggleUnlock then) and it's
--               at sequence 255.
-- Sequence 255: ACTOR1 (the Scion in the Waking Sands) rewards it (scene 3, a yield returning 1 when the reward is
--               taken), then the cutscene (scene 4, 0xF8402EFB, parameter 0; retail set UI8AH and UI8AL to 1 as it
--               began), and once it ends the quest is completed (the gil, 300 Storm Seals, the EXP) and the player
--               put at POPRANGE0 in the Waking Sands (retail's PrepareZoning and ActorSetPos, within a yalm of it.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ENLIST = 2
SCENE_REWARD = 3
SCENE_CUTSCENE = 4

-- Retail's flags (Bob's Chronofoil captures b8432613 and c99fe374.)
CUTSCENE_FLAGS = 0xF8402EFB
-- The company it enlists the player in (the client's script asks for this one.)
COMPANY = ORDER_OF_MAELSTROM
-- The UnlockLink retail turned on as the quest was accepted (ToggleUnlock 22, both captures.)
UNLOCK_GRAND_COMPANY = 22
WAKING_SANDS = 212

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The reward chosen in scene 3 (none: the Quest sheet's are fixed), kept until the cutscene after it ends.
reward = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
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
        play(player, SCENE_ENLIST, HIDE_HOTBAR)
        return
    elseif scene == SCENE_ENLIST then
        player:accept_quest(EVENT_ID)
        player:unlock(UNLOCK_GRAND_COMPANY)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            reward = results[2] or 0
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS, { 0 })
            player:set_quest_var(EVENT_ID, 0, 0x11) -- UI8AH and UI8AL, as retail set them
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:finish_event()
        player:change_territory_pop_range(WAKING_SANDS, POPRANGE0)
        player:finish_quest(EVENT_ID, reward)
        return
    end

    player:finish_event()
end

-- The enlistment's yield (GrandCompanyAssignment, its company in the results): the player joins it and the client
-- is told yes.
function onYield(scene, id, results, player)
    if scene == SCENE_ENLIST and current_scene == SCENE_ENLIST then
        player:set_grand_company(COMPANY)
        player:resume_event(scene, id, { 1 })
        return
    end
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
