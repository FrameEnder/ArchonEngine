-- Quest: Thanks a Million (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts; the client refuses it itself while QST_ACCEPT_CHECK is
--               accepted): sequence 1.
-- Sequence 1:   five spots to search, EOBJECT0..EOBJECT4: EOBJECTn plays scene n + 1 (empty), EVENT_ACTION_SEARCH (the
--               progress bar), then scene 100 - 2n when it's the one with ITEM0 or 99 - 2n when it isn't, all with
--               HIDE_HOTBAR | NO_DEFAULT_CAMERA. Only EOBJECT0's has anything in it (scene 100, the client's pop
--               message: an ambush), so EOBJECT0 has ITEM0 (on retail it's layout 3927161, the quest's todo location):
--               notice 0 [25004], UI8BH = 1, sequence 255; then ENEMY1 and ENEMY0 come out (layout IDs, drop-in layer
--               QB_SubSea105_001; killing them changes nothing). The others' scenes are empty.
-- Sequence 255: hand ITEM0 to ACTOR1 (scene 6, an NpcTrade: 1 if given), then the reward (scene 90, a yield). Retail
--               then sends FinishQuest for 404 too, with no rewards of its own (`mark_quest_completed`).

SCENE_OFFER = 0
SCENE_TRADE = 6
SCENE_FINISH = 90

NOTICE_FOUND = 0
ITEM_ICON = 25004 -- what retail's notice 0 carried
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

-- Retail sent FinishQuest for this one too, straight after this quest's (Bob's Chronofoil capture of 2026-10-21).
QUEST_ALSO_COMPLETED = 404
-- The spot with ITEM0 on retail (EOBJECT0's layout), in case the object's base ID isn't known.
SEARCH_SPOT = 3927161

-- Which of EOBJECT0..EOBJECT4 the target is (0..4), if it's one.
function spotIndex(target)
    for n, id in pairs({ [0] = EOBJECT0, [1] = EOBJECT1, [2] = EOBJECT2, [3] = EOBJECT3, [4] = EOBJECT4 }) do
        if id == BASE_ID then
            return n
        end
    end
    if target ~= nil and target.object_id == SEARCH_SPOT then
        return 0
    end
    return nil
end

-- The spot whose bar is running.
searching = nil

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local n = spotIndex(target)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif (sequence == 1 or sequence == 255) and n ~= nil then
        searching = n
        play(player, n + 1, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil then
        player:finish_event()
    elseif searching == 0 and player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, 100, OBJECT_FLAGS)
    else
        play(player, 99 - 2 * searching, OBJECT_FLAGS)
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
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
        end
    elseif searching ~= nil and scene == searching + 1 then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == 100 then
        searching = nil
        player:event_notice(EVENT_ID, NOTICE_FOUND, { ITEM_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1
        player:quest_sequence(EVENT_ID, 255)
        -- Retail's order: the scene, its end, then the ambush (each is brought out once at a time.)
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY1)
        player:spawn_quest_bnpc(ENEMY0)
        return
    else
        searching = nil
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
