-- Quest: Three for Three (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/889.json and `889-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   three places to search, each its empty scene, EVENT_ACTION_SEARCH's bar and its second scene (all
--               0x2001), then UI8AH (the count) up one, notice 0 [count, 3, the icon] and, with the third, sequence
--               255 with UI8BH, UI8BL and UI8CH 1 (the three items, as the client lists them then):
--               * EOBJECT1 (offered while UI8AL is 0 and BitFlag8 2 unset; scenes 5, 6): UI8AL 1, BitFlag8 2, ITEM1
--                 (UI8CL);
--               * EOBJECT2 (UI8BH 0, BitFlag8 3; scenes 7, 8): UI8BH 1, BitFlag8 3, ITEM2 (UI8DH);
--               * EOBJECT0 (BitFlag8 1; scenes 3, 4), guarded: walking into EVENTRANGE0 while UI8BL is under 3 plays
--                 scene 2 (its pop-up message) and brings out ENEMY0-2 (BNpcBase 726, 724, 725 at layouts 4290080,
--                 4290078, 4290079; drop-in layer QB_GaiUsb607_001 of roc_r1f1, retail's positions, level 30, 780
--                 HP); each one down counts in UI8BL. EOBJECT0's scene 3 is the client's warning while UI8BL is under
--                 3, and then there's no bar; with all three down, the bar and scene 4: BitFlag8 1, ITEM0 (UI8CH).
--               Both characters searched EOBJECT1, EOBJECT2, then EOBJECT0.
-- Sequence 255: ACTOR0 takes the three (scene 9, NpcTrade, a yield: 1 when handed over), then the cutscene and reward
--               (scene 10, 0xF8402EFB, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 2
SCENE_TRADE = 9
SCENE_REWARD = 10

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

-- The three places: their scenes, BitFlag8, what marks them done (byte, high nibble) and where their item is counted.
PLACES = {
    [EOBJECT1] = { scene = 5, flag = 2, done = { 0, false }, item = { 2, false } },
    [EOBJECT2] = { scene = 7, flag = 3, done = { 1, true }, item = { 3, true } },
    [EOBJECT0] = { scene = 3, flag = 1, item = { 2, true }, guarded = true },
}
NEEDED = 3
ENEMIES = { ENEMY0, ENEMY1, ENEMY2 }
-- The icon on the progress notices (Bob's Chronofoil captures of 2026-11-06.)
PROGRESS_ICON = 21003
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

function beaten(player)
    return getVar(player, 1, false) -- UI8BL
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The place being searched, while its bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local place = PLACES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and place ~= nil and not hasFlag(player, place.flag)
        and (place.done == nil or getVar(player, place.done[1], place.done[2]) == 0) then
        searching = place
        play(player, place.scene, QUIET_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 springs the ambush.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or beaten(player) >= #ENEMIES then
        player:finish_event()
        return
    end
    play(player, SCENE_AMBUSH, QUIET_FLAGS)
end

function onEventActionCast(player)
    if searching == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    play(player, searching.scene + 1, QUIET_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    for _, enemy in ipairs(ENEMIES) do
        if layout_id == enemy then
            setVar(player, 1, false, beaten(player) + 1)
            return
        end
    end
end

-- One place searched.
function searched(player, place)
    local count = getVar(player, 0, true) + 1 -- UI8AH
    player:event_notice(EVENT_ID, 0, { count, NEEDED, PROGRESS_ICON })
    if count >= NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH ITEM0, UI8BL ITEM1
        player:set_quest_var(EVENT_ID, 2, 0x10) -- UI8CH ITEM2
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, 0, true, count)
    if place.done ~= nil then
        setVar(player, place.done[1], place.done[2], 1)
    end
    setVar(player, place.item[1], place.item[2], getVar(player, place.item[1], place.item[2]) + 1)
    setFlag(player, place.flag)
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
    elseif scene == SCENE_AMBUSH then
        player:finish_event()
        -- Ones still out aren't brought out again.
        for _, enemy in ipairs(ENEMIES) do
            player:spawn_quest_bnpc(enemy)
        end
        return
    elseif searching ~= nil and scene == searching.scene then
        if not searching.guarded or beaten(player) >= #ENEMIES then
            player:event_action(EVENT_ACTION_SEARCH)
            return
        end
        searching = nil
    elseif searching ~= nil and scene == searching.scene + 1 then
        local place = searching
        searching = nil
        searched(player, place)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, CUTSCENE_FLAGS)
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
