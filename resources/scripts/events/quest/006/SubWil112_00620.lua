-- Quest: A Proper Burial (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/620.json).
-- Sequence 0:   ACTOR0 (Isembard) offers it (scene 0, a yield returning 1 when accepted; the client accepts it in it):
--               sequence 1 with ITEM0 (UI8BH 1; the client lists it as a key item from that).
-- Sequence 1:   ACTOR1 (Marques, scene 1): notice 0, sequence 2.
-- Sequence 2:   EOBJECT0, the gravesite (scene 2: the client opens the key item window); ITEM0 used on it (event type 8,
--               scene 3, which is empty): notice 1, sequence 3, the item gone. ACTOR1 meanwhile: scene 4.
-- Sequence 3:   EOBJECT1, the embalmed corpse (scene 5, then EVENT_ACTION_SEARCH's bar, then scene 6, all 0x2001):
--               notice 2, sequence 4. ACTOR1 meanwhile: scene 7.
-- Sequence 4:   ACTOR1 (scene 9): notice 3, sequence 5.
-- Sequence 5:   ACTOR3 (Ourcen, scene 12): notice 4, sequence 255. ACTOR1 meanwhile: scene 13.
-- Sequence 255: ACTOR0's reward (scene 16, a cutscene, retail's flags 0xF8402EFB; a yield returning 1 and the item
--               chosen.) ACTOR1 and ACTOR3 have lines (17, 18).
-- The client's IsAcceptEvent offers each sequence's NPCs in the order of the scenes after it, which is how the lines
-- above are matched; its other scenes (8, 10, 11, 14, 15: ACTOR2 and the corpse afterwards) are empty and retail
-- played none of them. Variables: none but ITEM0's (the client's other checks, UI8AL and BitFlag8 1, retail never set.)

SCENE_OFFER = 0
SCENE_MARQUES = 1
SCENE_GRAVESITE = 2
SCENE_ITEM_USED = 3
SCENE_SEARCH = 5
SCENE_SEARCHED = 6
SCENE_MARQUES_AFTER = 9
SCENE_OURCEN = 12
SCENE_REWARD = 16

-- Retail's flags (Bob's Chronofoil capture b8432613.)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
CUTSCENE_FLAGS = 0xF8402EFB

-- The lines the others have in each sequence: { [sequence] = { [base ID] = scene } }.
LINES = {
    [2] = { [ACTOR1] = 4 },
    [3] = { [ACTOR1] = 7 },
    [5] = { [ACTOR1] = 13 },
    [255] = { [ACTOR1] = 17, [ACTOR3] = 18 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local line = sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] or nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_MARQUES, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        play(player, SCENE_GRAVESITE, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        -- The corpse's scene, the bar (see sceneFinished), then its scene once that finishes.
        play(player, SCENE_SEARCH, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR1 then
        play(player, SCENE_MARQUES_AFTER, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR3 then
        play(player, SCENE_OURCEN, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- ITEM0 used on the gravesite.
function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS)
        return
    end
    player:finish_event()
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 3 then
        player:finish_event()
        return
    end
    play(player, SCENE_SEARCHED, OBJECT_FLAGS)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local sequence = player:get_quest_sequence(EVENT_ID)

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        end
    elseif scene == SCENE_MARQUES and sequence == 1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ITEM_USED then
        player:event_notice(EVENT_ID, 1, {})
        player:set_quest_var(EVENT_ID, 1, 0)
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_SEARCH then
        -- Searching shows a progress bar; the corpse's scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_SEARCHED then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_MARQUES_AFTER then
        player:event_notice(EVENT_ID, 3, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_OURCEN then
        player:event_notice(EVENT_ID, 4, {})
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
