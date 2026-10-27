-- Quest: Amalj'aa Wrong Places (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/622.json).
-- Sequence 0:   ACTOR0 (Isembard) offers it (scene 0, a yield returning 1 when accepted; the client accepts it in it):
--               sequence 1.
-- Sequence 1:   ACTOR1 (Thancred, scene 1): notice 0, sequence 2.
-- Sequence 2:   EOBJECT0, the queer leaflet (scene 2, then EVENT_ACTION_SEARCH's bar, then scene 3, all 0x2001):
--               notice 1 [icon], sequence 3 with ITEM0 (UI8BH 1). ACTOR1 meanwhile: scene 5.
-- Sequence 3:   ACTOR1 looks at it (scene 6, 0x2001: an NpcTrade, a yield returning 1 when handed over; then scene 7,
--               0x6001, he walks off): notice 2 [icon], sequence 4, the leaflet still yours.
-- Sequence 4:   ACTOR2 (Ourcen) takes it (scene 8, an NpcTrade, a yield; then scene 9): notice 3, sequence 5, the
--               variables cleared. ACTOR3 (Thancred, now by Ourcen) meanwhile: scene 10.
-- Sequence 5:   ACTOR3 (scene 13, 0x2001, he walks off): notice 4, sequence 255. ACTOR2 meanwhile: scene 14.
-- Sequence 255: ACTOR0's reward (scene 17, a yield: 1 and the item chosen.) ACTOR2: scene 18.
-- The client's IsAcceptEvent offers each sequence's NPCs in the order of the scenes after it, which is how the lines
-- above are matched; its other scenes (4, 11, 12, 15, 16, 19, 20: ACTOR4 and ACTOR5, the two by Ourcen) are empty and
-- retail played none of them.

SCENE_OFFER = 0
SCENE_THANCRED = 1
SCENE_LEAFLET = 2
SCENE_LEAFLET_FOUND = 3
SCENE_SHOW_THANCRED = 6
SCENE_THANCRED_LEAVES = 7
SCENE_OURCEN_TRADE = 8
SCENE_OURCEN_TAKES = 9
SCENE_THANCRED_AFTER = 13
SCENE_REWARD = 17

-- Retail's flags (Bob's Chronofoil capture b8432613.)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
LEAVING_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001

-- The leaflet's icon, as retail sent it with notices 1 and 2.
ITEM0_ICON = 25906

-- The lines the others have in each sequence: { [sequence] = { [base ID] = scene } }.
LINES = {
    [2] = { [ACTOR1] = 5 },
    [4] = { [ACTOR3] = 10 },
    [5] = { [ACTOR2] = 14 },
    [255] = { [ACTOR2] = 18 },
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
        play(player, SCENE_THANCRED, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        -- The leaflet's scene, the bar (see sceneFinished), then its scene once that finishes.
        play(player, SCENE_LEAFLET, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_SHOW_THANCRED, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_OURCEN_TRADE, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR3 then
        play(player, SCENE_THANCRED_AFTER, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SCENE_LEAFLET_FOUND, OBJECT_FLAGS)
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
    elseif scene == SCENE_THANCRED then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_LEAFLET then
        -- Searching shows a progress bar; the leaflet's scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_LEAFLET_FOUND then
        player:event_notice(EVENT_ID, 1, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 3)
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
    elseif scene == SCENE_SHOW_THANCRED then
        -- 1 when the leaflet was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_THANCRED_LEAVES, LEAVING_FLAGS)
            return
        end
    elseif scene == SCENE_THANCRED_LEAVES then
        player:event_notice(EVENT_ID, 2, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_OURCEN_TRADE then
        if results[1] == 1 then
            play(player, SCENE_OURCEN_TAKES, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_OURCEN_TAKES then
        player:event_notice(EVENT_ID, 3, {})
        player:set_quest_var(EVENT_ID, 1, 0)
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_THANCRED_AFTER then
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
