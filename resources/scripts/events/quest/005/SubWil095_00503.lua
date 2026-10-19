-- Quest: Give It to Me Raw (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-19) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 503):
-- Sequence 0:   ACTOR0 (the Drunken Stag) offers it (scene 0, a yield: QuestOffer and QuestAccepted in one, 1 when
--               accepted.) No variables.
-- Sequence 1:   three ore mounds, in any order (the client offers each while its BitFlag8 is unset: EOBJECT0 1,
--               EOBJECT1 2, EOBJECT2 3). Each plays its scene (1, 4 or 7, which only waits), then the gathering bar
--               (EVENT_ACTION_GATHER_SHORT, 2 s on retail), then its scene after the bar (2, 5 or 8, a pop-up
--               message), all with 0x2001; when the event has finished its marshlight comes out (ENEMY0, ENEMY1 or
--               ENEMY2, layout IDs, one per mound). Nothing changes until it's defeated: then its own variable
--               (UI8AL, UI8BH or UI8BL; the client stops counting that enemy at 1), the mound's BitFlag8, one more
--               ore (ITEM0, UI8CH) and one more mound done (UI8AH, todo 0 ticks at 3), and notice 0 [done, 3, icon].
--               The third clears the variables, moves the ores to UI8BH (3) and goes to 255.
-- Sequence 255: ACTOR0 takes the ores (trade scene 10, a yield: 1 when handed over) and rewards it (scene 11, a
--               yield: 1 and the chosen item.)
-- Scenes 3, 6, 9 and 12 are empty in the client's script and retail never played them.
-- The marshlights are layer QB_SubWil095_001 of the Western Thanalan drop-in, at retail's positions, level and HP.

SCENE_OFFER = 0
SCENE_TRADE = 10
SCENE_FINISH = 11
GATHER_SCENE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

-- Each mound: its scenes before and after the bar, its BitFlag8, its marshlight and the nibble that counts it.
MOUNDS = {
    [EOBJECT0] = { before = 1, after = 2, flag = 1, enemy = ENEMY0, byte = 0, high = false }, -- UI8AL
    [EOBJECT1] = { before = 4, after = 5, flag = 2, enemy = ENEMY1, byte = 1, high = true }, -- UI8BH
    [EOBJECT2] = { before = 7, after = 8, flag = 3, enemy = ENEMY2, byte = 1, high = false }, -- UI8BL
}
MOUNDS_NEEDED = 3

-- The progress pop-up after each marshlight, with the icon retail sends (Bob's Chronofoil capture of 2026-10-18.)
NOTICE_PROGRESS = 0
ORE_ICON = 20004

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
-- The mound being gathered from, while its bar runs.
gathering = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local mound = MOUNDS[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and mound ~= nil and not hasFlag(player, mound.flag) then
        -- The mound's first scene, the bar (see sceneFinished), then its scene once that finishes.
        gathering = mound
        play(player, mound.before, GATHER_SCENE_FLAGS)
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

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    local mound = nil
    for _, m in pairs(MOUNDS) do
        if m.enemy == layout_id then
            mound = m
        end
    end
    if mound == nil or getVar(player, mound.byte, mound.high) ~= 0 then
        return
    end

    local done = getVar(player, 0, true) + 1 -- UI8AH
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { done, MOUNDS_NEEDED, ORE_ICON })
    if done >= MOUNDS_NEEDED then
        -- Retail: everything cleared, the three ores in UI8BH.
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, MOUNDS_NEEDED)
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, 0, true, done)
    setVar(player, mound.byte, mound.high, 1)
    setVar(player, 2, true, getVar(player, 2, true) + 1) -- UI8CH: another ITEM0
    setFlag(player, mound.flag)
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
    elseif gathering ~= nil and scene == gathering.before then
        -- Gathering shows a progress bar; the mound's scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_GATHER_SHORT)
        return
    elseif gathering ~= nil and scene == gathering.after then
        -- Its marshlight comes out once the event is over, as on retail.
        local enemy = gathering.enemy
        gathering = nil
        player:finish_event()
        player:spawn_quest_bnpc(enemy)
        return
    elseif scene == SCENE_TRADE then
        -- 1 when the ores were handed over, nothing when the trade window was closed.
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
