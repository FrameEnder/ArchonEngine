-- Quest: Big Trouble in Little Ala Mhigo
--
-- Written from the client's quest script (kawari-questdump, 2026-10-31) and checked against Bob's Chronofoil capture
-- 20ef0e90 (2026-10-31, the whole quest and its battle: its fixture is tools/questkit/fixtures/782.json):
-- Sequence 0:   ACTOR0 (Gundobald) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene
--               (scene 90, 0xF8402EFB, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   five to see to, in any order (the client's IsAcceptEvent offers each while its nibble and flag are
--               unset):
--                 ACTOR1, scene 2 (0x6001): UI8AL = 1, BitFlag8 1
--                 ACTOR2, scene 3 (0x6001): UI8BH = 1, BitFlag8 2
--                 ACTOR3, scene 4 (0x6001): UI8BL = 1, BitFlag8 3
--                 EOBJECT0, scene 5 (0x2001, empty), EVENT_ACTION_SEARCH's bar, then scene 6 (0x2001, empty):
--                   UI8CH = 1, BitFlag8 4 and ITEM0 (UI8DL = 1)
--                 EOBJECT1, scene 7, the bar, then scene 8 (as EOBJECT0's): UI8CL = 1, BitFlag8 5 and ITEM1
--                   (UI8DH = 1)
--               UI8AH counts them, and each sends notice 0 [count, 5], the objects with the item's icon as well.
--               After the fifth retail cleared the variables, kept both items (UI8BH = UI8BL = 1: the client lists
--               them from there in sequence 2) and moved to sequence 2. ACTOR4 (Bertliana) meanwhile: scene 9.
-- Sequence 2:   ACTOR0 takes them (scene 10, NpcTrade in event mode: a yield, 1 when handed over), then scene 11:
--               notice 1, the variables cleared, sequence 3. ACTOR4: scene 12.
-- Sequence 3:   ACTOR5 (Wilred) asks whether to enter QUESTBATTLE0 (scene 13: YesNoQuestBattle, a yield, 1 for yes),
--               which takes the player to TERRITORYTYPE0 (306). Winning the battle sets sequence 255: that's the
--               battle's director (content/questbattle/58.lua), not this script. ACTOR4: scene 21. EOBJECT2 can't be
--               targeted (the client's IsTargetingPossible); which of scenes 14-20 belong to ACTOR6-10, the
--               bystanders the client accepts talks with, isn't known (retail's weren't recorded), so they get none.
-- Sequence 255: ACTOR0 rewards it (scene 22, a cutscene and a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 90
SCENE_TRADE = 10
SCENE_TRADED = 11
SCENE_ENTER_BATTLE = 13
SCENE_REWARD = 22
-- Bertliana's, in sequences 1, 2 and 3.
SCENE_BERTLIANA = { [1] = 9, [2] = 12, [3] = 21 }

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
TALK_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001 on retail
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_FOUND = 0
NOTICE_TRADED = 1

-- The five in sequence 1: their scene (and, for the objects, the scene after the bar), the nibble that counts them,
-- their flag and, for the objects, the item's nibble and its icon as retail's notice carried it (Bob's Chronofoil
-- capture 20ef0e90.)
TALKS = {
    { actor = ACTOR1, scene = 2, byte = 0, high = false, flag = 1 },
    { actor = ACTOR2, scene = 3, byte = 1, high = true, flag = 2 },
    { actor = ACTOR3, scene = 4, byte = 1, high = false, flag = 3 },
    { actor = EOBJECT0, scene = 5, searched = 6, byte = 2, high = true, flag = 4, item = { 3, false }, icon = 26151 },
    { actor = EOBJECT1, scene = 7, searched = 8, byte = 2, high = false, flag = 5, item = { 3, true }, icon = 30362 },
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
current_talk = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_talk = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence ~= nil and BASE_ID == ACTOR4 and SCENE_BERTLIANA[sequence] ~= nil then
        play(player, SCENE_BERTLIANA[sequence], HIDE_HOTBAR)
        return
    elseif sequence == 1 then
        for _, talk in ipairs(TALKS) do
            if BASE_ID == talk.actor and not hasFlag(player, talk.flag) then
                current_talk = talk
                play(player, talk.scene, talk.searched and OBJECT_FLAGS or TALK_FLAGS)
                return
            end
        end
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == ACTOR5 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
        return
    end
    player:finish_event()
end

-- The search bar on an object finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or current_talk == nil or current_talk.searched == nil then
        current_talk = nil
        player:finish_event()
        return
    end
    play(player, current_talk.searched, OBJECT_FLAGS)
end

-- One of the five seen to.
function found(player, talk)
    local done = getVar(player, 0, true) + 1
    if talk.icon ~= nil then
        player:event_notice(EVENT_ID, NOTICE_FOUND, { done, #TALKS, talk.icon })
    else
        player:event_notice(EVENT_ID, NOTICE_FOUND, { done, #TALKS })
    end
    if done >= #TALKS then
        clearVars(player)
        -- Both items kept, where the client lists them in sequence 2.
        setVar(player, 1, true, 1)
        setVar(player, 1, false, 1)
        player:quest_sequence(EVENT_ID, 2)
    else
        setVar(player, 0, true, done)
        setVar(player, talk.byte, talk.high, 1)
        if talk.item ~= nil then
            setVar(player, talk.item[1], talk.item[2], 1)
        end
        setFlag(player, talk.flag)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif current_talk ~= nil and scene == current_talk.scene then
        if current_talk.searched ~= nil then
            -- Searching the object: the bar, then the scene after it (see onEventActionCast.)
            player:event_action(EVENT_ACTION_SEARCH)
            return
        end
        local talk = current_talk
        current_talk = nil
        found(player, talk)
    elseif current_talk ~= nil and scene == current_talk.searched then
        local talk = current_talk
        current_talk = nil
        found(player, talk)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:event_notice(EVENT_ID, NOTICE_TRADED, {})
        clearVars(player)
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
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
