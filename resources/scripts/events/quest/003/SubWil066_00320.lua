-- Quest: Way Down in the Hole (Ul'dah)
--
-- Written from retail (Bob's Chronofoil capture of 2026-10-18, questkit replay 320) and Sapphire's quest battle; not
-- yet checked against the client's script (no kawari-questdump of it yet), so the NPCs are compared by the ENpc IDs
-- retail showed rather than the quest's ACTOR names:
-- Sequence 0:   the issuer offers it (scene 0, a yield returning 1 when accepted), then scene 1; accepted after it.
-- Sequence 1:   the foreman (scene 2) hands you four of the item (UI8CL = 4, notice 0 with its icon 24051), sequence 2.
-- Sequence 2:   give one to each of four miners, in any order (each a yield, scenes 3, 6, 9 or 12, returning 1, then
--               the next scene with 0x6001): UI8AH counts them (notice 1 [count, 4]), UI8CL counts the items left,
--               and each miner has their own variable and BitFlag8 (the first UI8AL and flag 1, the second UI8BH and
--               flag 2, the third UI8BL and flag 3; the fourth's isn't seen: the last one clears everything and moves
--               to sequence 3.)
-- Sequence 3:   the foreman's cutscene (scene 16): notice 2, sequence 4.
-- Sequence 4:   the guard at the mine (scene 17, a yield returning 1) sends you into quest battle 18 (territory 248);
--               winning it moves the quest to sequence 5.
-- Sequence 5:   back in Central Thanalan, the aftermath cutscene (scene 26, an enter-territory event): sequence 6.
-- Sequence 6:   the foreman (scene 27, 0x6001): notice 5, sequence 255.
-- Sequence 255: Momodi in Ul'dah rewards it (scene 28, a yield with the chosen item.)

ISSUER = 1001578       -- layout 3775589
FOREMAN = 1002947      -- layout 3815997
GUARD = 1002955        -- layout 3817007, the way into the mine
MOMODI = 1001353       -- layout 3146810
QUEST_BATTLE = QUESTBATTLE0 or 18
QUEST_BATTLE_TERRITORY = TERRITORYTYPE0 or 248

-- The four miners: their ENpc, the scenes (offer, then hand-over), and the variable and BitFlag8 marking them done.
MINERS = {
    [1002948] = { offer = 3, given = 4, byte = 0, high = false, flag = 1 }, -- layout 3841638, UI8AL
    [1002949] = { offer = 6, given = 7, byte = 1, high = true, flag = 2 },  -- layout 3841641, UI8BH
    [1002950] = { offer = 9, given = 10, byte = 1, high = false, flag = 3 }, -- layout 3841642, UI8BL
    [1002951] = { offer = 12, given = 13, byte = 2, high = true, flag = 4 }, -- layout 3841645, UI8CH (a guess)
}
MINERS_NEEDED = 4
ITEMS_GIVEN = 4

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_FOREMAN = 2
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

    if sequence == nil and BASE_ID == ISSUER then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == FOREMAN then
        play(player, SCENE_FOREMAN, HIDE_HOTBAR)
    elseif sequence == 2 and miner ~= nil and not hasBitFlag8(player, miner.flag) then
        current_miner = miner
        play(player, miner.offer, MINER_FLAGS)
    elseif sequence == 3 and BASE_ID == FOREMAN then
        play(player, SCENE_FOREMAN_CUTSCENE, CUTSCENE_FLAGS)
    elseif sequence == 4 and BASE_ID == GUARD then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == FOREMAN then
        play(player, SCENE_REPORT, MINER_GIVEN_FLAGS)
    elseif sequence == 255 and BASE_ID == MOMODI then
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
            player:join_quest_battle(QUEST_BATTLE, QUEST_BATTLE_TERRITORY)
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
