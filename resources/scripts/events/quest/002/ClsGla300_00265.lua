-- Quest: The Rematch (the Gladiators' level 30 quest)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil capture
-- accc33a9 (2026-11-06, the whole quest on a Gladiator, its quest battle won): its fixture is
-- tools/questkit/fixtures/265.json.
-- Sequence 0:   ACTOR0 (Mylla, in Ul'dah - Steps of Thal) offers it (scene 0, a yield: the client accepts it there,
--               returning 1): sequence 1.
-- Sequence 1:   ACTOR1 (Wymond, in Ul'dah - Steps of Nald, scene 1, with its menu): notice 0, sequence 2.
-- Sequence 2:   the two places in Southern Thanalan, in any order (the client offers each while its BitFlag8 is
--               unset: EOBJECT0 flag 1, EOBJECT1 flag 2): its empty scene (2 or 5, 0x2001), EVENT_ACTION_SEARCH's bar
--               (2 s on retail), then its scene after (3 or 6, 0x2001, the pop-up message); as that ends its three
--               (ENEMY0-2 at EOBJECT0, ENEMY3-5 at EOBJECT1: layers QB_ClsGla300_001 and _002 of the wil_w1f4 drop-in,
--               retail's positions, level 30 and 585 HP) come out for the player, an ambush to fight off (retail's
--               kills changed nothing.) Each place: its key item (EOBJECT0 ITEM0 in UI8BL, EOBJECT1 ITEM1 in UI8CH:
--               the client's GetEventItems), its notice (1 [ITEM0's icon], 2 [ITEM1's icon]), UI8AL one more and its
--               flag; EOBJECT0 also sets UI8AH = 1, as retail did. Both: the variables cleared but UI8BH = 1 (ITEM0)
--               and UI8BL = 1 (ITEM1, as the client reads them from sequence 3), sequence 3. Retail went to EOBJECT0
--               first; EOBJECT1 first marks itself in UI8BH (a guess: retail never showed it.)
-- Sequence 3:   ACTOR0 is shown the key items (scene 8, NpcTrade in event mode: a yield, 1 when shown) and talks
--               (scene 9): notice 3 [ITEM0's icon], sequence 4; both key items stay.
-- Sequence 4:   ACTOR2 (in Eastern Thanalan) is shown them (scene 10, NpcTrade) and asks (scene 11, YesNoQuestBattle);
--               1 enters QUESTBATTLE0 (23) in TERRITORYTYPE0 (257), and it can be tried again after a loss. Winning it
--               moves to sequence 255 with every variable 0 (content/questbattle/23.lua.) ACTOR3, ACTOR0 and ACTOR6
--               have lines meanwhile (12, 14, 16); ACTOR4's, ACTOR5's and EOBJECT2's scenes are empty.
-- Sequence 255: ACTOR0 rewards it (scene 19, a cutscene and a yield: QuestReward, 1 and the optional item picked;
--               closing the window instead leaves UI8A 0x11, as retail does: docs/src/fork/quests.md.)

SCENE_OFFER = 0
SCENE_WYMOND = 1
SCENE_SHOW_MYLLA = 8
SCENE_MYLLA = 9
SCENE_SHOW = 10
SCENE_BATTLE = 11
SCENE_REWARD = 19

-- Retail's flags (Bob's Chronofoil capture accc33a9.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
EVENT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

-- The key items' icons, as the notices carried them on retail (Bob's Chronofoil capture accc33a9.)
ITEM0_ICON = 25918
ITEM1_ICON = 20662
NOTICE_MYLLA_SHOWN = 3
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Sequence 2's places: their scenes, BitFlag8, notice and icon, the three that come out, where their key item is
-- held (byte, high nibble) and where it's marked done.
PLACES = {
    [EOBJECT0] = { scene = 2, after = 3, flag = 1, notice = 1, icon = ITEM0_ICON, enemies = { ENEMY0, ENEMY1, ENEMY2 },
        item_byte = 1, item_high = false, done_byte = 0, done_high = true },
    [EOBJECT1] = { scene = 5, after = 6, flag = 2, notice = 2, icon = ITEM1_ICON, enemies = { ENEMY3, ENEMY4, ENEMY5 },
        item_byte = 2, item_high = true, done_byte = 1, done_high = true },
}
PLACES_NEEDED = 2

-- The other lines, by sequence: the client's scene for each NPC it offers then.
LINES = {
    [4] = { [ACTOR3] = 12, [ACTOR0] = 14, [ACTOR6] = 16 },
}

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

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The place being searched in this event.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local place = PLACES[BASE_ID]
    searching = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_WYMOND, HIDE_HOTBAR)
    elseif sequence == 2 and place ~= nil and not hasFlag(player, place.flag) then
        searching = place
        play(player, place.scene, EVENT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR0 then
        play(player, SCENE_SHOW_MYLLA, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_SHOW, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The search bar finished: the place's scene after it.
function onEventActionCast(player)
    if current_scene ~= nil or searching == nil or player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, searching.after, EVENT_FLAGS)
end

-- One place searched: its key item, notice and marks; both: sequence 3.
function searched(player, place)
    setVar(player, place.item_byte, place.item_high, 1)
    setVar(player, place.done_byte, place.done_high, 1)
    setFlag(player, place.flag)
    local count = getVar(player, 0, false) + 1
    setVar(player, 0, false, count) -- UI8AL: how many
    player:event_notice(EVENT_ID, place.notice, { place.icon })
    if count >= PLACES_NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:set_quest_var(EVENT_ID, 1, 0x11) -- UI8BH = 1 (ITEM0), UI8BL = 1 (ITEM1)
        player:quest_sequence(EVENT_ID, 3)
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
    elseif scene == SCENE_WYMOND then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif searching ~= nil and scene == searching.scene then
        -- The bar, then the scene after it (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif searching ~= nil and scene == searching.after then
        local place = searching
        searching = nil
        searched(player, place)
        player:finish_event()
        -- Its three come out as the event ends (ones still out aren't brought out again.)
        for _, enemy in ipairs(place.enemies) do
            player:spawn_quest_bnpc(enemy)
        end
        return
    elseif scene == SCENE_SHOW_MYLLA then
        if results[1] == 1 then
            play(player, SCENE_MYLLA, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_MYLLA then
        player:event_notice(EVENT_ID, NOTICE_MYLLA_SHOWN, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_SHOW then
        if results[1] == 1 then
            play(player, SCENE_BATTLE, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
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
