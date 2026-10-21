-- Quest: Double Dealing (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts; the client refuses it itself while QST_ACCEPT_CHECK is
--               accepted): sequence 1.
-- Sequence 1:   ACTOR1 won't talk (scene 1); an emote on ACTOR1 (the Quest sheet listens for one; retail's was emote
--               12) plays scene 2: notice 0, sequence 2.
-- Sequence 2:   walking into EVENTRANGE0 plays scene 16 (the client's pop message; nothing changes). ACTOR2..ACTOR4, in
--               any order: ACTORn plays scene 3n + 1 (empty), EVENT_ACTION_ (35, the progress bar), then scene 3n + 2
--               (they run off), then notice 1 with [count, 3]. UI8AL counts them, BitFlag8 n - 1 marks ACTORn; the
--               third clears the variables and moves to sequence 3. ACTOR1 has a line meanwhile (scene 18).
-- Sequence 3:   ACTOR1 (scene 24): notice 2, sequence 4.
-- Sequence 4:   EOBJECT2: scene 28 (empty), EVENT_ACTION_PROCESS_MIDDLE, scene 29: ITEM0 (UI8BH = 1), notice 3
--               [25919], sequence 5.
-- Sequence 5:   hand ITEM0 to ACTOR5 (scene 33, an NpcTrade: 1 if given), then scene 34: ITEM1 instead (UI8BL = 1),
--               notice 4 [25919], sequence 255.
-- Sequence 255: hand ITEM1 to ACTOR0 (scene 36, an NpcTrade), then the reward (scene 37, a yield: 1 completes it.)
-- All scenes as retail played them: the talks with NPCs and objects in sequences 2-4 with HIDE_HOTBAR |
-- NO_DEFAULT_CAMERA, the rest with HIDE_HOTBAR. EOBJECT0, EOBJECT1 and ENEMY0/1 weren't used on retail.

SCENE_OFFER = 0
SCENE_WONT_TALK = 1
SCENE_EMOTE = 2
SCENE_RANGE = 16
SCENE_ACTOR1_WAITING = 18
SCENE_ACTOR1_REPORT = 24
SCENE_EOBJECT = 28 -- empty in the client's script: the progress bar follows
SCENE_EOBJECT_DONE = 29
SCENE_TRADE_ACTOR5 = 33
SCENE_ACTOR5_DONE = 34
SCENE_TRADE_ACTOR0 = 36
SCENE_FINISH = 37

NOTICE_EMOTE = 0
NOTICE_TALKED = 1
NOTICE_REPORTED = 2
NOTICE_ITEM0 = 3
NOTICE_ITEM1 = 4
-- The notices' argument for the items, as retail sent it (both items.)
ITEM_ICON = 25919

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one. BitFlag8 n is bit 8 - n of byte 5.
function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

function hasBitFlag8(player, n)
    return (player:get_quest_var(EVENT_ID, 5) & (1 << (8 - n))) ~= 0
end

function setBitFlag8(player, n)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (1 << (8 - n)))
end

function clearVars(player)
    for b = 0, 5 do
        player:set_quest_var(EVENT_ID, b, 0)
    end
end

-- Which of ACTOR2..ACTOR4 BASE_ID is (2..4), if it's one.
function witness()
    for n, id in pairs({ [2] = ACTOR2, [3] = ACTOR3, [4] = ACTOR4 }) do
        if id == BASE_ID then
            return n
        end
    end
    return nil
end

-- The witness whose bar is running.
talking_to = nil

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local n = witness()

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_WONT_TALK, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1_WAITING, HIDE_HOTBAR)
    elseif sequence == 2 and n ~= nil and not hasBitFlag8(player, n - 1) then
        talking_to = n
        play(player, 3 * n + 1, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1_REPORT, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == EOBJECT2 then
        play(player, SCENE_EOBJECT, OBJECT_FLAGS)
    elseif sequence == 5 and BASE_ID == ACTOR5 then
        play(player, SCENE_TRADE_ACTOR5, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE_ACTOR0, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    if player:get_quest_sequence(EVENT_ID) == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_EMOTE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) == 2 and arg == EVENTRANGE0 then
        play(player, SCENE_RANGE, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 2 and talking_to ~= nil then
        play(player, 3 * talking_to + 2, OBJECT_FLAGS)
    elseif sequence == 4 then
        play(player, SCENE_EOBJECT_DONE, OBJECT_FLAGS)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_EMOTE then
        player:event_notice(EVENT_ID, NOTICE_EMOTE, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif talking_to ~= nil and scene == 3 * talking_to + 1 then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_)
        return
    elseif talking_to ~= nil and scene == 3 * talking_to + 2 then
        local n = talking_to
        talking_to = nil
        local count = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
        player:event_notice(EVENT_ID, NOTICE_TALKED, { count, 3 })
        if count >= 3 then
            clearVars(player)
            player:quest_sequence(EVENT_ID, 3)
        else
            setVar(player, 0, false, count)
            setBitFlag8(player, n - 1)
        end
    elseif scene == SCENE_ACTOR1_REPORT then
        player:event_notice(EVENT_ID, NOTICE_REPORTED, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_EOBJECT then
        player:event_action(EVENT_ACTION_PROCESS_MIDDLE)
        return
    elseif scene == SCENE_EOBJECT_DONE then
        player:event_notice(EVENT_ID, NOTICE_ITEM0, { ITEM_ICON })
        setVar(player, 1, true, 1)
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_TRADE_ACTOR5 then
        if results[1] == 1 then
            play(player, SCENE_ACTOR5_DONE, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACTOR5_DONE then
        player:event_notice(EVENT_ID, NOTICE_ITEM1, { ITEM_ICON })
        clearVars(player)
        setVar(player, 1, false, 1)
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE_ACTOR0 then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_FINISH then
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
