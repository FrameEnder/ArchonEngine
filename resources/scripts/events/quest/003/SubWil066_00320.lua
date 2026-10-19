-- Quest: Way Down in the Hole (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 320; the quest battle from Sapphire's):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: the client's QuestOffer, 1 when accepted; the client itself refuses
--               while QST_ACCEPT_CHECK is accepted), then scene 1, where the client accepts it; accepted after it.
-- Sequence 1:   the foreman, ACTOR1 (scene 2, only talk), hands you four ITEM0 (UI8CL = 4, notice 0 with its icon
--               24051), sequence 2. (The client lists ITEM0 from UI8BH in sequence 1 and from UI8CL in sequence 2.)
-- Sequence 2:   give one to each of ACTOR2..ACTOR5, in any order (each a hand-over window, scenes 3, 6, 9 or 12,
--               a yield returning 1 when handed over, then the next scene, 4, 7, 10 or 13, with 0x6001, where they
--               run off): UI8AH counts them (todo 1 ticks at 4; notice 1 [count, 4]), UI8CL counts the items left,
--               and each has their own variable and BitFlag8, the client offering them while both are unset: ACTOR2
--               UI8AL and flag 1, ACTOR3 UI8BH and flag 2, ACTOR4 UI8BL and flag 3, ACTOR5 UI8CH and flag 4 (the
--               last one done clears everything and moves to sequence 3, so retail never shows the fourth's.)
--               The foreman can be talked to meanwhile (scene 15, only talk; from the client's script, not retail.)
-- Sequence 3:   the foreman's cutscene (scene 16): notice 2, sequence 4.
-- Sequence 4:   ACTOR6 at the mine (scene 17, a yield: the client's YesNoQuestBattle for QUESTBATTLE0, 1 for yes; the
--               client offers it while UI8AL and BitFlag8 1 are unset) sends you into QUESTBATTLE0 (TERRITORYTYPE0);
--               winning it moves the quest to sequence 5. ACTOR7..ACTOR12 and EOBJECT0 are also offered in this
--               sequence, but the client's script has no scene for them (18-25 are empty): the event just ends.
-- Sequence 5:   back in TERRITORYTYPE1, the aftermath cutscene (scene 26, an enter-territory event): sequence 6.
-- Sequence 6:   the foreman (scene 27, 0x6001): notice 5, sequence 255.
-- Sequence 255: ACTOR13 (Momodi) in Ul'dah rewards it (scene 28, a yield: QuestReward, 1 and the chosen item; the
--               client shows the unlocks itself, the server grants REWARD0 and REWARD_LEVE.)

-- The four ITEM0 recipients: the scenes (hand-over, then given) and the variable and BitFlag8 marking them done,
-- from the client's IsAcceptEvent.
MINERS = {
    [ACTOR2] = { offer = 3, given = 4, byte = 0, high = false, flag = 1 }, -- UI8AL
    [ACTOR3] = { offer = 6, given = 7, byte = 1, high = true, flag = 2 }, -- UI8BH
    [ACTOR4] = { offer = 9, given = 10, byte = 1, high = false, flag = 3 }, -- UI8BL
    [ACTOR5] = { offer = 12, given = 13, byte = 2, high = true, flag = 4 }, -- UI8CH
}
MINERS_NEEDED = 4
ITEMS_GIVEN = 4

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_FOREMAN = 2
SCENE_FOREMAN_WAITING = 15
SCENE_FOREMAN_CUTSCENE = 16
SCENE_ENTER_BATTLE = 17
SCENE_AFTERMATH = 26
SCENE_REPORT = 27
SCENE_FINISH = 28

NOTICE_ITEMS = 0
NOTICE_MINER = 1
NOTICE_ALL_MINERS = 2
NOTICE_REPORT = 5
ITEM_ICON = 24051 -- what retail's notice 0 carried

-- Scene flags as retail sent them.
MINER_GIVEN_FLAGS = INVINCIBLE | HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x6001
MINER_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA                   -- 0x2001
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR                         -- 0xF8402EFB
REWARD_FLAGS = SET_BASE | HIDE_HOTBAR | DISABLE_SKIP            -- 0xF8482EFB

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one. BitFlag8 n is bit 8 - n of byte 5.
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

function setBitFlag8(player, n)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (1 << (8 - n)))
end

function hasBitFlag8(player, n)
    return player:get_quest_var(EVENT_ID, 5) & (1 << (8 - n)) ~= 0
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The miner whose scenes are playing.
current_miner = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local miner = MINERS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_FOREMAN, HIDE_HOTBAR)
    elseif sequence == 2 and miner ~= nil and getVar(player, miner.byte, miner.high) == 0
        and not hasBitFlag8(player, miner.flag) then
        current_miner = miner
        play(player, miner.offer, MINER_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_FOREMAN_WAITING, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_FOREMAN_CUTSCENE, CUTSCENE_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR6 and getVar(player, 0, false) == 0 and not hasBitFlag8(player, 1) then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == ACTOR1 then
        play(player, SCENE_REPORT, MINER_GIVEN_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR13 then
        play(player, SCENE_FINISH, REWARD_FLAGS)
    else
        player:finish_event()
    end
end

-- Back in Central Thanalan after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 5 then
        play(player, SCENE_AFTERMATH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function minerDone(player, miner)
    local count = getVar(player, 0, true) + 1 -- UI8AH
    player:event_notice(EVENT_ID, NOTICE_MINER, { count, MINERS_NEEDED })
    if count >= MINERS_NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:quest_sequence(EVENT_ID, 3)
        return
    end
    setVar(player, 0, true, count)
    setVar(player, miner.byte, miner.high, 1)
    setVar(player, 2, false, ITEMS_GIVEN - count) -- UI8CL: the items left
    setBitFlag8(player, miner.flag)
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
    elseif scene == SCENE_FOREMAN then
        setVar(player, 2, false, ITEMS_GIVEN) -- UI8CL = 4
        player:event_notice(EVENT_ID, NOTICE_ITEMS, { ITEM_ICON })
        player:quest_sequence(EVENT_ID, 2)
    elseif current_miner ~= nil and scene == current_miner.offer then
        if results[1] == 1 then
            play(player, current_miner.given, MINER_GIVEN_FLAGS)
            return
        end
        current_miner = nil
    elseif current_miner ~= nil and scene == current_miner.given then
        minerDone(player, current_miner)
        current_miner = nil
    elseif scene == SCENE_FOREMAN_CUTSCENE then
        player:event_notice(EVENT_ID, NOTICE_ALL_MINERS)
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:quest_sequence(EVENT_ID, 6)
    elseif scene == SCENE_REPORT then
        player:event_notice(EVENT_ID, NOTICE_REPORT)
        player:quest_sequence(EVENT_ID, 255)
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
