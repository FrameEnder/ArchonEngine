-- Quest: Fire in the Gloom (Gridania)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-24) and Bob's Chronofoil capture of the quest
-- on retail (2026-10-24, 3ec702a8: New Gridania, the Central Shroud, the Tam-Tara Deepcroft with Duty Support, back to
-- Mother Miounne):
-- Sequence 0:   ACTOR0 (Mother Miounne, territory 132) offers it (scene 0, a yield: 1 accepts), then the cutscene
--               (scene 1): sequence 1.
-- Sequence 1:   ACTOR1 (the Gods' Quiver bow, territory 148) (scene 2, which shows Tam-Tara's unlock image): the
--               Tam-Tara Deepcroft (INSTANCEDUNGEON0, InstanceContent 2) is unlocked (ActorControlSelf 131 before the
--               notice), notice 0, sequence 2. The client's IsAcceptEvent wants UI8AL 0 and BitFlag8 1 unset for it;
--               retail set no variables. Meanwhile ACTOR2-6 have a line each (scenes 3-7; retail played ACTOR4's, 5).
-- Sequence 2:   clearing the Tam-Tara Deepcroft: retail moved it on as the player left the cleared duty (notice 1, no
--               variables, sequence 255), which the server tells the script with onDutyCleared. Meanwhile ACTOR2-6
--               (scenes 8-12) and ACTOR1 (scene 13) have a line each.
-- Sequence 255: ACTOR0 rewards it (scene 14, a cutscene and a yield: 1 completes it).

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BOW = 2
SCENE_SEQ1_LINES = 3 -- ACTOR2..ACTOR6: 3..7
SCENE_SEQ2_LINES = 8 -- ACTOR2..ACTOR6: 8..12
SCENE_BOW_AFTER = 13
SCENE_FINISH = 14

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played the cutscenes

-- Notices (Bob's Chronofoil capture of 2026-10-24): one per todo, none with arguments.
NOTICE_BOW = 0
NOTICE_TAM_TARA = 1

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- ACTOR2..ACTOR6's place (0..4), or nil.
function bystander(base_id)
    local actors = { ACTOR2, ACTOR3, ACTOR4, ACTOR5, ACTOR6 }
    for i, id in ipairs(actors) do
        if base_id == id then
            return i - 1
        end
    end
    return nil
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local which = bystander(BASE_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_BOW, HIDE_HOTBAR)
    elseif sequence == 1 and which ~= nil then
        play(player, SCENE_SEQ1_LINES + which, HIDE_HOTBAR)
    elseif sequence == 2 and which ~= nil then
        play(player, SCENE_SEQ2_LINES + which, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_BOW_AFTER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- Leaving a duty the player cleared (its InstanceContent ID). Not an event of the quest's: nothing to finish.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 2 and content_id == INSTANCEDUNGEON0 then
        player:event_notice(EVENT_ID, NOTICE_TAM_TARA, {})
        player:quest_sequence(EVENT_ID, 255)
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
    elseif scene == SCENE_BOW then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, NOTICE_BOW, {})
        player:quest_sequence(EVENT_ID, 2)
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
