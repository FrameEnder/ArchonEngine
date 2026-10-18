-- Quest: Leia's Legacy (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest
-- (questkit replay):
-- Sequence 0:   ACTOR0 (Luquelot) offers and accepts the quest in one scene (scene 0, returns 1 when accepted.)
-- Sequence 1:   ask ACTOR1 (Leodaire), ACTOR2 (Bernard) and ACTOR3 (Ewmond) about the egg, in any order (scenes 1, 2
--               and 3.) UI8AL counts them and each sets its BitFlag8 (1 to 3, in UI8F) so the client stops offering it;
--               each sends notice 0 with [count, 3]. The third moves straight to sequence 2 with the variables cleared.
-- Sequence 2:   report to ACTOR0 (cutscene 4), which sends notice 1 and moves to sequence 3.
-- Sequence 3:   ACTOR4 (Kuplu Kopo) offers the quest battle (scene 5, YesNoQuestBattle returns 1.) QUESTBATTLE0 takes
--               place in TERRITORYTYPE0; winning it moves the quest to sequence 4 (content/questbattle/14.lua.)
-- Sequence 4:   ACTOR4 again (scene 11) gives ITEM0, the egg (UI8BH = 1, the client lists it as a key item), with
--               notice 3 [ITEM0's icon], and moves to sequence 5.
-- Sequence 5:   hand the egg to ACTOR0 (trade scene 15, 1 when handed over, then cutscene 16): notice 4, sequence 255,
--               variables cleared.
-- Sequence 255: ACTOR0 and the reward (scene 18.)

SCENE_OFFER = 0
SCENE_REPORT = 4
SCENE_BATTLE = 5
SCENE_EGG = 11
SCENE_TRADE = 15
SCENE_TRADE_OK = 16
SCENE_REWARD = 18

-- The three to ask: their scene and BitFlag8 index.
WITNESSES = {
    [ACTOR1] = { scene = 1, flag = 1 },
    [ACTOR2] = { scene = 2, flag = 2 },
    [ACTOR3] = { scene = 3, flag = 3 },
}
WITNESSES_NEEDED = 3

NOTICE_ASKED = 0
NOTICE_REPORTED = 1
NOTICE_EGG = 3
NOTICE_RETURNED = 4
-- Cutscenes play as Sapphire plays them.
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays its cutscenes (Bob's Chronofoil capture of 2026-10-14)

-- ITEM0's icon, as retail sends it with notice 3.
ITEM0_ICON = 25252

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

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local witness = WITNESSES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and witness ~= nil and not hasFlag(player, witness.flag) then
        play(player, witness.scene, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_REPORT, CUTSCENE_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR4 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        -- Retail's flags here: the cutscene set without hiding event objects, and INVINCIBLE (0xF8406EF3.)
        play(player, SCENE_EGG, (SET_BASE & ~INVIS_EOBJ) | HIDE_HOTBAR | INVINCIBLE)
    elseif sequence == 5 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function asked(player, flag)
    local count = getVar(player, 0, false) + 1 -- UI8AL
    player:event_notice(EVENT_ID, NOTICE_ASKED, { count, WITNESSES_NEEDED })
    if count >= WITNESSES_NEEDED then
        clearVars(player)
        player:quest_sequence(EVENT_ID, 2)
    else
        setVar(player, 0, false, count)
        setFlag(player, flag)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local sequence = player:get_quest_sequence(EVENT_ID)
    local witness = WITNESSES[BASE_ID]

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif sequence == 1 and witness ~= nil and scene == witness.scene then
        asked(player, witness.flag)
    elseif scene == SCENE_REPORT then
        player:event_notice(EVENT_ID, NOTICE_REPORTED)
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_EGG then
        setVar(player, 1, true, 1) -- UI8BH: ITEM0, the egg
        player:event_notice(EVENT_ID, NOTICE_EGG, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_TRADE then
        -- 1 when the egg was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_TRADE_OK then
        player:event_notice(EVENT_ID, NOTICE_RETURNED)
        clearVars(player)
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
