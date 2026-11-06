-- Quest: Factual Folklore (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/927.json and `927-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down itself while
--               QST_ACCEPT_CHECK_01 or _02 is in the journal), then scene 1 (QuestAccepted and the key item's how-to):
--               sequence 1 with ITEM0 (UI8BL = 1.)
-- Sequence 1:   EOBJECT0: its empty scene 2, EVENT_ACTION_SEARCH's bar, then scene 3 (its pop-up message; both
--               0x2001); as it ends ENEMY0 (BNpcBase 2196, layout 4293214; drop-in layer QB_GaiUsb904_001 of
--               roc_r1f1, retail's position, level 39, 14283 HP) comes out for the player; searched again it comes out
--               again if it's gone (Bob's first character searched three times.) ITEM0 used on it (ENEMY1 is its
--               BNpcBase): its statuses 22 and 62 and a fifth of its health (`event_item_effects.json`, the
--               server's), and the event ends straight away (`onEventItem`, as retail.) ENEMY0 down: notice 0 [the
--               icon], sequence 2 with ITEM1 (UI8BH = 1) and ITEM0 gone.
-- Sequence 2:   ACTOR0 takes ITEM1 (scene 4, NpcTrade, a yield: 1 when handed over) and talks (scene 5): notice 1
--               [the icon], sequence 3 with three ITEM2 (UI8CH = 3.)
-- Sequence 3:   ACTOR1, 2 and 3, in any order, each once (its counter UI8AL, UI8BH or UI8BL and BitFlag8 1-3): its
--               trade (scenes 6, 8, 10: NpcTrade, a yield) and its talk (7, 9, 11): UI8AH (the count) up one, an ITEM2
--               less, notice 2 [count, 3]; the third: sequence 4, every variable 0.
-- Sequence 4:   ACTOR0 (scene 12): notice 3 [the icon], sequence 255 with ITEM3 (UI8BH = 1.)
-- Sequence 255: ACTOR4 takes it (scene 13, NpcTrade) and rewards it (scene 14, a yield: QuestReward, 1 and the
--               optional item picked; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_SEARCH = 2
SCENE_SEARCHED = 3
SCENE_TRADE_ITEM1 = 4
SCENE_AFTER_ITEM1 = 5
SCENE_ACTOR0 = 12
SCENE_TRADE_ITEM3 = 13
SCENE_REWARD = 14

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- The icons on notices 0, 1 and 3 (Bob's Chronofoil captures of 2026-11-06.)
ITEM1_ICON = 25152
NOTICE_ICON = 24352

-- Sequence 3's three: their trade scene (the talk follows it), counter (byte, high nibble) and BitFlag8.
HANDOUTS = {
    [ACTOR1] = { scene = 6, byte = 0, high = false, flag = 1 },
    [ACTOR2] = { scene = 8, byte = 1, high = true, flag = 2 },
    [ACTOR3] = { scene = 10, byte = 1, high = false, flag = 3 },
}
NEEDED = 3
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getVar(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return value >> 4
    end
    return value & 0x0F
end

function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

function clearVars(player)
    for byte = 0, 5 do
        player:set_quest_var(EVENT_ID, byte, 0)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
handing = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local handout = HANDOUTS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_SEARCH, QUIET_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE_ITEM1, HIDE_HOTBAR)
    elseif sequence == 3 and handout ~= nil and getVar(player, handout.byte, handout.high) == 0
        and not hasFlag(player, handout.flag) then
        handing = handout
        play(player, handout.scene, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_TRADE_ITEM3, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar at EOBJECT0 finished.
function onEventActionCast(player)
    if current_scene ~= nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_SEARCHED, QUIET_FLAGS)
end

-- ITEM0 on ENEMY0: what it does is the server's (event_item_effects.json); the event has nothing more.
function onEventItem(item_id, target, player)
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) == 1 and layout_id == ENEMY0 then
        player:event_notice(EVENT_ID, 0, { ITEM1_ICON })
        clearVars(player)
        setVar(player, 1, true, 1) -- UI8BH: ITEM1
        player:quest_sequence(EVENT_ID, 2)
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
        setVar(player, 1, false, 1) -- UI8BL: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_SEARCH then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_SEARCHED then
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY0)
        return
    elseif scene == SCENE_TRADE_ITEM1 then
        if results[1] == 1 then
            play(player, SCENE_AFTER_ITEM1, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_AFTER_ITEM1 then
        player:event_notice(EVENT_ID, 1, { NOTICE_ICON })
        clearVars(player)
        setVar(player, 2, true, 3) -- UI8CH: three ITEM2
        player:quest_sequence(EVENT_ID, 3)
    elseif handing ~= nil and scene == handing.scene then
        if results[1] == 1 then
            play(player, handing.scene + 1, HIDE_HOTBAR)
            return
        end
        handing = nil
    elseif handing ~= nil and scene == handing.scene + 1 then
        local handout = handing
        handing = nil
        local count = getVar(player, 0, true) + 1 -- UI8AH
        player:event_notice(EVENT_ID, 2, { count, NEEDED })
        if count >= NEEDED then
            clearVars(player)
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, true, count)
            setVar(player, handout.byte, handout.high, 1)
            setVar(player, 2, true, getVar(player, 2, true) - 1)
            setFlag(player, handout.flag)
        end
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, 3, { NOTICE_ICON })
        setVar(player, 1, true, 1) -- UI8BH: ITEM3
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE_ITEM3 then
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
