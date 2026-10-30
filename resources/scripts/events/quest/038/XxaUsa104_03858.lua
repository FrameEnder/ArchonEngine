-- Quest: Forest Friend
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/3858.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down itself while
--               QST_ACCEPT_CHECK, the old Forest Friend (718), is accepted), then scene 1, where the client accepts
--               it: sequence 1.
-- Sequence 1:   ACTOR1 (Imedia) talks (scene 2): notice 0, sequence 2.
-- Sequence 2:   three todos, in any order; UI8AH counts the ones done:
--                 todo 1: defeat ENEMY0 (a Shroudbee swarm, BNpcBase 179): UI8BL = 1, notice 1
--                 todo 2: defeat ENEMY1 (a giant gnat, BNpcBase 7): UI8BH = 1, notice 2
--                 todo 3: search the three brownie brushes (EOBJECT0-2: scenes 3 and 4, 5 and 6, 7 and 8, all 0x2001;
--                         the first only waits, then EVENTACTION0's bar, then the second): UI8AL counts them, ITEM0
--                         in UI8CH counts the same, BitFlag8 1-3 marks each (the client offers each while its flag is
--                         unset), notice 3 [count, 3, ITEM0's icon]
--               The client counts the kills while UI8BL / UI8BH are 0. With all three done retail cleared all but
--               ITEM0, which it moved to UI8BH (= 3; the client lists ITEM0 from UI8CH in sequence 2 and from UI8BH in
--               255), and moved to sequence 255.
-- Sequence 255: ACTOR2 (Komuxio) takes ITEM0 (scene 9, NpcTrade in event mode: a yield, 1 when handed over) and
--               rewards it (scene 10, a yield: QuestReward, 1.) Retail then completed the old Forest Friend (718) too:
--               FinishQuest 3858, then 718.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_IMEDIA = 2
SCENE_TRADE = 9
SCENE_REWARD = 10
FLAGS_BRUSH = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_IMEDIA = 0
NOTICE_SWARM = 1
NOTICE_GNAT = 2
NOTICE_BRUSH = 3
BRUSHES_NEEDED = 3
TODOS_NEEDED = 3
-- ITEM0's icon, as retail's notices carried it (Bob's Chronofoil capture 5fcc970c.)
ITEM0_ICON = 25406
-- The quest this one replaced, completed with it (retail's FinishQuest 718 right after 3858's.)
QUEST_ALSO_COMPLETED = 718

-- The brushes: their scenes (before and after the bar) and flag.
BRUSHES = {
    { eobj = EOBJECT0, scene = 3, found = 4, flag = 1 },
    { eobj = EOBJECT1, scene = 5, found = 6, flag = 2 },
    { eobj = EOBJECT2, scene = 7, found = 8, flag = 3 },
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

-- A todo of sequence 2 done: counted in UI8AH; with all three, only ITEM0 is kept, moved to UI8BH (where the client
-- looks for it in sequence 255), and the quest goes to 255.
function todoDone(player)
    local done = getVar(player, 0, true) + 1
    if done >= TODOS_NEEDED then
        local item = getVar(player, 2, true)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, item)
        player:quest_sequence(EVENT_ID, 255)
    else
        setVar(player, 0, true, done)
    end
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    searching = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_IMEDIA, HIDE_HOTBAR)
        return
    elseif sequence == 2 then
        for _, brush in ipairs(BRUSHES) do
            if BASE_ID == brush.eobj and not hasFlag(player, brush.flag) then
                searching = brush
                play(player, brush.scene, FLAGS_BRUSH)
                return
            end
        end
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEventActionCast(player)
    if searching == nil then
        player:finish_event()
        return
    end
    play(player, searching.found, FLAGS_BRUSH)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    if base_id == ENEMY0 and getVar(player, 1, false) == 0 then
        player:event_notice(EVENT_ID, NOTICE_SWARM, {})
        setVar(player, 1, false, 1)
        todoDone(player)
    elseif base_id == ENEMY1 and getVar(player, 1, true) == 0 then
        player:event_notice(EVENT_ID, NOTICE_GNAT, {})
        setVar(player, 1, true, 1)
        todoDone(player)
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
    elseif scene == SCENE_IMEDIA then
        player:event_notice(EVENT_ID, NOTICE_IMEDIA, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif searching ~= nil and scene == searching.scene then
        -- The brush's scene only waits: then the bar (see onEventActionCast.)
        player:event_action(EVENTACTION0)
        return
    elseif searching ~= nil and scene == searching.found then
        local found = getVar(player, 0, false) + 1
        player:event_notice(EVENT_ID, NOTICE_BRUSH, { found, BRUSHES_NEEDED, ITEM0_ICON })
        setVar(player, 0, false, found)
        setVar(player, 2, true, found) -- ITEM0 count in UI8CH
        setFlag(player, searching.flag)
        searching = nil
        if found >= BRUSHES_NEEDED then
            todoDone(player)
        end
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
