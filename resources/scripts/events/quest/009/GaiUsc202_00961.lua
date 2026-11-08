-- Quest: Representing the Representative (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (18a89a32, 4bcbd425 and de5beba7, each the whole quest; fixtures
-- tools/questkit/fixtures/961.json, 961-18a89a32.json, 961-de5beba7.json):
-- Sequence 0:   ACTOR0 (Ceana) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Skyfryn) (scene 2): notice 0 [ITEM0's icon], sequence 2 with UI8BH = 1 (ITEM0 held.)
-- Sequence 2:   ACTOR0 (scene 3): notice 1, sequence 3. ACTOR1 has a line meanwhile (4).
-- Sequence 3:   ACTOR2 (Mimidoa) takes ITEM0 (scene 5, NpcTrade in event mode: a yield, 1 when handed over), then
--               scene 6: notice 2 [ITEM1's icon], sequence 255 (UI8BH = 1 is now ITEM1, as the client's GetEventItems
--               reads it in sequence 255.) ACTOR1 has a line meanwhile (7, and in sequence 255.)
-- Sequence 255: ACTOR0 takes ITEM1 (scene 8, NpcTrade: a yield, 1) and rewards it (scene 9, a yield: QuestReward, 1.)
--               ACTOR2 has a line meanwhile (10.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_SKYFRYN = 2
SCENE_CEANA = 3
SCENE_MIMIDOA_TRADE = 5
SCENE_MIMIDOA_TAKEN = 6
SCENE_TRADE = 8
SCENE_REWARD = 9

NOTICE_SKYFRYN = 0
NOTICE_CEANA = 1
NOTICE_MIMIDOA = 2
-- The items' icons, as retail's notices carried them (all three captures.)
ITEM0_ICON = 21901
ITEM1_ICON = 26153
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Lines by sequence and NPC.
LINES = {
    [2] = { [ACTOR1] = 4 },
    [3] = { [ACTOR1] = 7 },
    [255] = { [ACTOR1] = 7, [ACTOR2] = 10 },
}

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
        play(player, SCENE_SKYFRYN, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_CEANA, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_MIMIDOA_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
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
    elseif scene == SCENE_SKYFRYN then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0 held
        player:event_notice(EVENT_ID, NOTICE_SKYFRYN, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_CEANA then
        player:event_notice(EVENT_ID, NOTICE_CEANA, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_MIMIDOA_TRADE then
        if results[1] == 1 then
            play(player, SCENE_MIMIDOA_TAKEN, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_MIMIDOA_TAKEN then
        player:event_notice(EVENT_ID, NOTICE_MIMIDOA, { ITEM1_ICON })
        player:quest_sequence(EVENT_ID, 255) -- UI8BH: ITEM1 now
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
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
