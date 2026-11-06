-- Quest: The Face of Thal (the Gladiators' level 20 quest)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil capture
-- 941895d2 (2026-11-06, the whole quest on a Gladiator, its quest battle won): its fixture is
-- tools/questkit/fixtures/263.json.
-- Sequence 0:   ACTOR0 (Mylla, in Ul'dah - Steps of Thal) offers it (scene 0, a yield: the client accepts it there,
--               returning 1): sequence 1, with ITEM0 (UI8BH = 1; the client lists it from there.)
-- Sequence 1:   ACTOR1 (the injured gladiator, in Central Thanalan) takes ITEM0 (scene 1, NpcTrade in event mode: a
--               yield, 1 when handed over) and talks (scene 2): notice 0, ITEM0 gone, sequence 2.
-- Sequence 2:   two things, in any order (the client's todos: UI8AL >= 2, UI8BH >= 1):
--               * ACTOR2 (Etfled, scene 4) and ACTOR3 (Roundelph, scene 5), each once (BitFlag8 1 and 2, flag 1 being
--                 UI8F's top bit): UI8AL one more, notice 1 [UI8AL, 2]; the second also sets UI8AH = 1, as retail
--                 did;
--               * EOBJECT0 (the point of ambush, BitFlag8 3): its empty scene 6 (0x2001), EVENT_ACTION_SEARCH_SHORT's
--                 bar (2 s on retail), then scene 7: ITEM1 found, notice 2 [ITEM1's icon].
--               Both done: every variable 0 but UI8BL = 1 (ITEM1), sequence 3. Retail talked first, so the point
--               moved it on at once; found first, it waits in UI8BH (= 1) and BitFlag8 3 for the talks.
-- Sequence 3:   ACTOR1 is shown ITEM1 (scene 10, NpcTrade in event mode) and talks (scene 11): notice 3 [ITEM1's
--               icon], sequence 4; ITEM1 stays (UI8BL = 1.)
-- Sequence 4:   ACTOR4 (Wymond) takes ITEM1 (scene 13, NpcTrade) and talks (scene 14): notice 4, sequence 5, every
--               variable 0.
-- Sequence 5:   ACTOR0 (scene 17, a cutscene: retail's 0xF8482EFB): notice 5, sequence 6.
-- Sequence 6:   ACTOR5 (Mylla, in Western Thanalan, scene 18): YesNoQuestBattle; 1 enters QUESTBATTLE0 (21) in
--               TERRITORYTYPE0 (255), and it can be tried again after a loss. Winning it moves to sequence 255
--               (content/questbattle/21.lua.) ACTOR0 back in Ul'dah has a line meanwhile (scene 19.)
-- Sequence 255: ACTOR0 rewards it (scene 22, a cutscene and a yield: QuestReward, 1 and the optional item picked;
--               closing the window instead leaves UI8A 0x11, as retail does: docs/src/fork/quests.md.)
-- The client's scenes follow its IsAcceptEvent order; the gladiator's lines in sequences 2 and 4 are LINES below. The
-- rest (3, 8, 12, 15, 20, 21) are empty.

SCENE_OFFER = 0
SCENE_TRADE = 1
SCENE_GLADIATOR = 2
SCENE_POINT = 6
SCENE_POINT_FOUND = 7
SCENE_SHOW = 10
SCENE_SHOWN = 11
SCENE_WYMOND_TRADE = 13
SCENE_WYMOND = 14
SCENE_MYLLA = 17
SCENE_BATTLE = 18
SCENE_REWARD = 22

-- Retail's flags (Bob's Chronofoil capture 941895d2.)
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
MYLLA_CUTSCENE_FLAGS = 0xF8482EFB
EVENT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

NOTICE_TRADED = 0
NOTICE_TALKED = 1
NOTICE_FOUND = 2
NOTICE_SHOWN = 3
NOTICE_WYMOND = 4
NOTICE_MYLLA = 5
-- ITEM1's icon, as the notices carried it on retail (Bob's Chronofoil capture 941895d2.)
ITEM1_ICON = 54434
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Sequence 2's two to talk to: their scene and BitFlag8.
WITNESSES = {
    [ACTOR2] = { scene = 4, flag = 1 },
    [ACTOR3] = { scene = 5, flag = 2 },
}
WITNESSES_NEEDED = 2
POINT_FLAG = 3

-- The other lines, by sequence: the client's scene for each NPC it offers then.
LINES = {
    [2] = { [ACTOR1] = 9 },
    [4] = { [ACTOR1] = 16 },
    [6] = { [ACTOR0] = 19 },
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

function clearVars(player)
    for byte = 0, 5 do
        player:set_quest_var(EVENT_ID, byte, 0)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The witness being talked to in this event.
witness = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local talked_to = WITNESSES[BASE_ID]
    witness = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and talked_to ~= nil and getVar(player, 0, false) < WITNESSES_NEEDED
        and not hasFlag(player, talked_to.flag) then
        witness = talked_to
        play(player, talked_to.scene, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 and getVar(player, 1, true) < 1 and not hasFlag(player, POINT_FLAG) then
        play(player, SCENE_POINT, EVENT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_SHOW, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        play(player, SCENE_WYMOND_TRADE, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR0 then
        play(player, SCENE_MYLLA, MYLLA_CUTSCENE_FLAGS)
    elseif sequence == 6 and BASE_ID == ACTOR5 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The search bar at the point of ambush finished: the scene after it.
function onEventActionCast(player)
    if current_scene ~= nil or player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, SCENE_POINT_FOUND, EVENT_FLAGS)
end

-- Sequence 2 is done once both witnesses are talked to and the point is found.
function checkSequence2(player)
    if getVar(player, 0, false) >= WITNESSES_NEEDED and getVar(player, 1, true) >= 1 then
        clearVars(player)
        setVar(player, 1, false, 1) -- UI8BL = 1: ITEM1
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
            setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_GLADIATOR, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_GLADIATOR then
        player:event_notice(EVENT_ID, NOTICE_TRADED, {})
        setVar(player, 1, true, 0) -- ITEM0 handed over
        player:quest_sequence(EVENT_ID, 2)
    elseif witness ~= nil and scene == witness.scene then
        local talked = getVar(player, 0, false) + 1
        setVar(player, 0, false, talked)
        setFlag(player, witness.flag)
        witness = nil
        player:event_notice(EVENT_ID, NOTICE_TALKED, { talked, WITNESSES_NEEDED })
        if talked >= WITNESSES_NEEDED then
            setVar(player, 0, true, 1) -- UI8AH = 1, as retail set it with the second talk
        end
        checkSequence2(player)
    elseif scene == SCENE_POINT then
        -- The bar, then the scene after it (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH_SHORT)
        return
    elseif scene == SCENE_POINT_FOUND then
        player:event_notice(EVENT_ID, NOTICE_FOUND, { ITEM1_ICON })
        setVar(player, 1, true, 1) -- UI8BH = 1: the point found
        setFlag(player, POINT_FLAG)
        checkSequence2(player)
    elseif scene == SCENE_SHOW then
        if results[1] == 1 then
            play(player, SCENE_SHOWN, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_SHOWN then
        player:event_notice(EVENT_ID, NOTICE_SHOWN, { ITEM1_ICON })
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_WYMOND_TRADE then
        if results[1] == 1 then
            play(player, SCENE_WYMOND, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_WYMOND then
        player:event_notice(EVENT_ID, NOTICE_WYMOND, {})
        clearVars(player) -- ITEM1 handed over
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_MYLLA then
        player:event_notice(EVENT_ID, NOTICE_MYLLA, {})
        player:quest_sequence(EVENT_ID, 6)
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
