-- Quest: Unsolved Mystery (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/618.json).
-- Sequence 0:   ACTOR0 (Isembard) offers it (scene 0, a yield returning 1 when accepted), then scene 1 (the client
--               accepts it there): sequence 1. No variables.
-- Sequence 1:   the three ripe corpses, in any order (the client offers each while its own variable is 0 and its
--               BitFlag8 unset: ACTOR1 UI8BL and 1, ACTOR2 UI8AL and 2, ACTOR3 UI8BH and 3). Each plays its scene
--               (2, 5 or 8, which only waits), then the gathering bar (EVENT_ACTION_GATHER_SHORT), then its scene after
--               the bar (3, 6 or 9), all with 0x2001; then its variable and flag, one more corpse done (UI8AH), one more
--               ITEM0 (UI8CH) and notice 0 [done, 3, icon]. The third clears the variables, moves the items to UI8BH (3)
--               and goes to 255.
--               Walking into EVENTRANGE0 brings out ENEMY0 and ENEMY1 (layout IDs, owned by the player, naming the
--               range as their trigger) with scene 10, 0x2001 (the client's ScenarioMessage, which it skips when the
--               two are already out for you); killing them changes nothing.
-- Sequence 255: ACTOR0 takes the items (trade scene 19, a yield: 1 when handed over) and rewards it (scene 20, a yield
--               returning 1.)
-- The client's other scenes (4, 7, 11-18, 21-27: the event objects around the corpses and the enemies) are empty and
-- retail played none of them.
-- The two ambushers are layer QB_SubWil110_001 of the Eastern Thanalan drop-in, at retail's positions, level and HP.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 10
SCENE_TRADE = 19
SCENE_FINISH = 20
GATHER_SCENE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

-- Each corpse: its scenes before and after the bar, its BitFlag8 and the nibble that marks it done.
CORPSES = {
    [ACTOR1] = { before = 2, after = 3, flag = 1, byte = 1, high = false }, -- UI8BL
    [ACTOR2] = { before = 5, after = 6, flag = 2, byte = 0, high = false }, -- UI8AL
    [ACTOR3] = { before = 8, after = 9, flag = 3, byte = 1, high = true }, -- UI8BH
}
CORPSES_NEEDED = 3

-- The progress pop-up after each corpse, with the icon retail sends (Bob's Chronofoil capture b8432613.)
NOTICE_PROGRESS = 0
ITEM0_ICON = 26024

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

-- BitFlag8 n is bit 8 - n of UI8F.
function hasFlag(player, flag)
    return player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1)) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The corpse being gathered from, while its bar runs.
gathering = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local corpse = CORPSES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and corpse ~= nil and getVar(player, corpse.byte, corpse.high) == 0
        and not hasFlag(player, corpse.flag) then
        -- The corpse's first scene, the bar (see sceneFinished), then its scene once that finishes.
        gathering = corpse
        play(player, corpse.before, GATHER_SCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if gathering == nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, gathering.after, GATHER_SCENE_FLAGS)
end

-- Walking into EVENTRANGE0 springs the ambush (one of each at a time: nothing new while they're out.)
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    player:spawn_quest_bnpc(ENEMY0)
    player:spawn_quest_bnpc(ENEMY1)
    play(player, SCENE_AMBUSH, GATHER_SCENE_FLAGS)
end

-- One corpse searched.
function gathered(player, corpse)
    local done = getVar(player, 0, true) + 1 -- UI8AH
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { done, CORPSES_NEEDED, ITEM0_ICON })
    if done >= CORPSES_NEEDED then
        -- Retail: everything cleared, the three items in UI8BH.
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, CORPSES_NEEDED)
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, 0, true, done)
    setVar(player, corpse.byte, corpse.high, 1)
    setVar(player, 2, true, getVar(player, 2, true) + 1) -- UI8CH: another ITEM0
    setFlag(player, corpse.flag)
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
    elseif gathering ~= nil and scene == gathering.before then
        -- Gathering shows a progress bar; the corpse's scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_GATHER_SHORT)
        return
    elseif gathering ~= nil and scene == gathering.after then
        local corpse = gathering
        gathering = nil
        gathered(player, corpse)
    elseif scene == SCENE_TRADE then
        -- 1 when the items were handed over, nothing when the trade window was closed.
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
