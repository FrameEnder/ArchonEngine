-- Quest: It's Probably Pirates (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-23) and Bob's Chronofoil capture of the quest
-- on retail (2026-10-23, 700cc294: Limsa Lominsa, Sastasha with Duty Support, back to Baderon):
-- Sequence 0:   ACTOR0 (Baderon, territory 128) offers it (scene 0, a yield: 1 accepts), then the cutscene (scene 1): sequence 1.
-- Sequence 1:   ACTOR1 (V'mellpa, territory 129) (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (the seasoned adventurer, territory 138) (scene 3, which shows the beginner's guide): notice 1, unlock REWARD_BEGINNER_01 (247), sequence 3.
--               ACTOR1 has one line meanwhile (scene 4).
-- Sequence 3:   ACTOR3 (Yellowjacket, territory 138) (scene 5): Sastasha (INSTANCEDUNGEON0, InstanceContent 4) is
--               unlocked (ActorControlSelf 131 before the notice), notice 2, sequence 4. Meanwhile ACTOR2 (scene 6),
--               ACTOR4-9 (scenes 7-12) and ACTOR1 (scene 13) have a line each.
-- Sequence 4:   clearing Sastasha: retail moved it on as the player left the cleared duty by its exit (notice 3, no
--               variables, sequence 255; the client's IsAcceptDirectorResult wants INSTANCEDUNGEON0 in this sequence),
--               which the server tells the script with onDutyCleared. Meanwhile ACTOR4-9 (scenes 14-19) and ACTOR3
--               (scene 20) have a line each.
-- Sequence 255: ACTOR0 rewards it (scene 21, a cutscene and a yield: 1 completes it); retail then marks quest 675 (the
--               same quest for players who didn't start in Limsa Lominsa, QST_ACCEPT_CHECK) completed too.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_VMELLPA = 2
SCENE_ADVENTURER = 3
SCENE_VMELLPA_AFTER = 4
SCENE_YELLOWJACKET = 5
SCENE_ADVENTURER_AFTER = 6
SCENE_SAILORS_SEQ3 = 7 -- ACTOR4..ACTOR9: 7..12
SCENE_VMELLPA_SEQ3 = 13
SCENE_SAILORS_SEQ4 = 14 -- ACTOR4..ACTOR9: 14..19
SCENE_YELLOWJACKET_AFTER = 20
SCENE_FINISH = 21

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail played the cutscenes

-- Notices (Bob's Chronofoil capture of 2026-10-23): one per todo, none with arguments.
NOTICE_VMELLPA = 0
NOTICE_ADVENTURER = 1
NOTICE_YELLOWJACKET = 2
NOTICE_SASTASHA = 3

-- Retail sent FinishQuest for this one too, straight after this quest's (Bob's Chronofoil capture of 2026-10-23).
QUEST_ALSO_COMPLETED = 675

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- ACTOR4..ACTOR9's place among the sailors (0..5), or nil.
function sailor(base_id)
    local sailors = { ACTOR4, ACTOR5, ACTOR6, ACTOR7, ACTOR8, ACTOR9 }
    for i, id in ipairs(sailors) do
        if base_id == id then
            return i - 1
        end
    end
    return nil
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local which = sailor(BASE_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_VMELLPA, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_ADVENTURER, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_VMELLPA_AFTER, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        play(player, SCENE_YELLOWJACKET, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ADVENTURER_AFTER, HIDE_HOTBAR)
    elseif sequence == 3 and which ~= nil then
        play(player, SCENE_SAILORS_SEQ3 + which, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_VMELLPA_SEQ3, HIDE_HOTBAR)
    elseif sequence == 4 and which ~= nil then
        play(player, SCENE_SAILORS_SEQ4 + which, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR3 then
        play(player, SCENE_YELLOWJACKET_AFTER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_FINISH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- Leaving a duty the player cleared (its InstanceContent ID). Not an event of the quest's: nothing to finish.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 4 and content_id == INSTANCEDUNGEON0 then
        player:event_notice(EVENT_ID, NOTICE_SASTASHA, {})
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
    elseif scene == SCENE_VMELLPA then
        player:event_notice(EVENT_ID, NOTICE_VMELLPA, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ADVENTURER then
        player:event_notice(EVENT_ID, NOTICE_ADVENTURER, {})
        player:unlock(REWARD_BEGINNER_01)
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_YELLOWJACKET then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, NOTICE_YELLOWJACKET, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
