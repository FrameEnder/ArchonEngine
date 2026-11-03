-- Quest: The Things We Do for Cheese
--
-- Written from the client's quest script (kawari-questdump, 2026-11-03) and checked against Bob's Chronofoil captures
-- 44df02fc and c9145fdc (2026-11-03, the whole quest each, with Brayflox's Longstop through Duty Support and the Duty
-- Finder: its fixtures are tools/questkit/fixtures/832.json and 832-c9145fdc.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the client
--               accepts it: sequence 1.
-- Sequence 1:   ACTOR1 talks (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 talks (scene 3: the dungeon's unlock image and the "new duty" log line): INSTANCEDUNGEON0
--               unlocked, notice 1, sequence 3.
-- Sequence 3:   talking to ACTOR2 again plays scene 4 (not seen on retail). Clearing the duty: retail moved the
--               quest on as the player left it by its exit (notice 2, sequence 4), which the server tells the script
--               with onDutyCleared; the player came back where they had queued (both runs).
-- Sequence 4:   ACTOR2's cutscene (scene 5, 0xF8402EFB): ITEM0 (UI8BH = 1), notice 3 [ITEM0's icon], sequence 255.
-- Sequence 255: ACTOR0 takes it (scene 6, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 7, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_UNLOCKED = 3
SCENE_WAITING = 4
SCENE_AFTER_DUTY = 5
SCENE_TRADE = 6
SCENE_REWARD = 7

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

NOTICE_ACTOR1 = 0
NOTICE_UNLOCKED = 1
NOTICE_CLEARED = 2
NOTICE_AFTER_DUTY = 3
-- ITEM0's icon, as retail's notice carried it (Bob's Chronofoil capture 44df02fc.)
ITEM0_ICON = 24451

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
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_UNLOCKED, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_WAITING, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_AFTER_DUTY, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Leaving a duty the player cleared (its InstanceContent ID). Not an event of the quest's: nothing to finish.
function onDutyCleared(content_id, player)
    if player:get_quest_sequence(EVENT_ID) == 3 and content_id == INSTANCEDUNGEON0 then
        player:event_notice(EVENT_ID, NOTICE_CLEARED, {})
        player:quest_sequence(EVENT_ID, 4)
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
        player:event_notice(EVENT_ID, NOTICE_ACTOR1, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_UNLOCKED then
        player:unlock_content(INSTANCEDUNGEON0)
        player:event_notice(EVENT_ID, NOTICE_UNLOCKED, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_AFTER_DUTY then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        player:event_notice(EVENT_ID, NOTICE_AFTER_DUTY, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
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
