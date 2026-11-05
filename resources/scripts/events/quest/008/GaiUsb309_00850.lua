-- Quest: It Was a Very Good Year
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b4ce5e3f (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/850.json), d65126c9
-- (an Arcanist, `850-d65126c9.json`) and 811eb9ff (a White Mage, `850-811eb9ff.json`):
-- Sequence 0:   ACTOR0 (Shamani Lohmani) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1,
--               where the client accepts it: sequence 1, with ITEM0 (UI8BH = 1; the client lists it from there.)
-- Sequence 1:   ACTOR1 (Drest) takes ITEM0 (scene 2, NpcTrade in event mode: a yield, 1 when handed over) and talks
--               (scene 3): notice 0, ITEM0 gone, sequence 2.
-- Sequence 2:   EOBJECT0 (the destination): its scene (4, 0x2001, which only waits), EVENT_ACTION_WAITING_SHOR's bar
--               (row 11), then scene 5 (0x2001, the client's pop-up message); as the event ends ENEMY0 comes out, the
--               player's own (layer QB_GaiUsb309_001 of the Eastern La Noscea drop-in, at retail's position, level
--               and HP; timeline GaiUsb309Enemy_1910.) Its death: notice 1 [ITEM1's icon], ITEM1 (UI8BH = 1, which
--               the client reads as ITEM1 while UI8AL is 0), sequence 3.
-- Sequence 3:   ACTOR0 takes ITEM1 (scene 6, NpcTrade in event mode) and the cutscene (scene 7, 0xF8402EFB) gives
--               ITEM2: notice 2 [ITEM2's icon], UI8BH = 1 (ITEM2 now), sequence 255.
-- Sequence 255: ACTOR2 (Wheiskaet) takes ITEM2 (scene 8, NpcTrade in event mode) and rewards it (scene 9, a yield:
--               QuestReward, 1.) Retail then completed A Final Ignominy (854) too, with no rewards (FinishQuest 850,
--               then 854, all three runs): In the Company of Heroes, next, is offered only while 854 isn't in the
--               journal (its QST_ACCEPT_CHECK.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_DREST_TRADE = 2
SCENE_DREST = 3
SCENE_DESTINATION = 4
SCENE_DESTINATION_DONE = 5
SCENE_SHAMANI_TRADE = 6
SCENE_SHAMANI = 7
SCENE_TRADE = 8
SCENE_REWARD = 9
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

NOTICE_DREST = 0
NOTICE_DEFEATED = 1
NOTICE_SHAMANI = 2
-- The key items' icons the notices carried on retail (Bob's Chronofoil capture b4ce5e3f.)
ITEM1_ICON = 22417
ITEM2_ICON = 25057
-- What retail set UI8A to when a reward window was closed (Bob's capture b766138c, quest 840; 414 and 680.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did (Bob's Chronofoil captures of 2026-11-05.)
QUEST_ALSO_COMPLETED = 854

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
        play(player, SCENE_DREST_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        play(player, SCENE_DESTINATION, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_SHAMANI_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The waiting bar at the destination finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SCENE_DESTINATION_DONE, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, { ITEM1_ICON })
    player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM1
    player:quest_sequence(EVENT_ID, 3)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_DREST_TRADE then
        if results[1] == 1 then
            play(player, SCENE_DREST, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_DREST then
        player:event_notice(EVENT_ID, NOTICE_DREST, {})
        player:set_quest_var(EVENT_ID, 1, 0) -- ITEM0 handed over
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_DESTINATION then
        -- The bar, then scene 5 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING_SHOR)
        return
    elseif scene == SCENE_DESTINATION_DONE then
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY0)
        return
    elseif scene == SCENE_SHAMANI_TRADE then
        if results[1] == 1 then
            play(player, SCENE_SHAMANI, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_SHAMANI then
        player:event_notice(EVENT_ID, NOTICE_SHAMANI, { ITEM2_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM2 now
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
