-- Quest: Salvaging the Scene (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest.
-- The Quest sheet doesn't name this quest's scenes; the numbers come from the client's script and Sapphire's.
-- Sequence 0:   ACTOR0 (Armelle) offers the quest (scene 0, returns 1 when accepted; the client itself refuses it while
--               QST_ACCEPT_CHECK isn't complete), then explains it (scene 1, which shows the quest as accepted.)
-- Sequence 1:   search the two fallen cargoes, EOBJECT0 and EOBJECT1, in either order (EVENT_ACTION_SEARCH, then scene
--               2 or 3.) Each gives one ITEM0 (counted in UI8BH; the client lists it as a key item from that), counts in
--               UI8AL and sets its BitFlag8 (1 or 2, in UI8F) so the client stops offering it, and sends notice 0 with
--               [count, 2, ITEM0's icon], the progress pop-up. The second moves straight to sequence 255 with the count
--               and flags cleared (the items stay.)
--               Walking into EVENTRANGE0 brings out ENEMY0, a lumber toad (a layout ID, in the Central Shroud drop-ins)
--               with scene 22's "ambush" message; it doesn't count for anything. The other objects around the wreck
--               (EOBJECT2 to EOBJECT6) can be examined but have nothing to show.
-- Sequence 255: hand both ITEM0 to ACTOR1 (Keitha) (scene 23, the trade window; scene 24 once they're handed over, with
--               the reward.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 22
SCENE_TRADE = 23
SCENE_TRADE_OK = 24

-- Each fallen cargo: its scene and BitFlag8 index.
CARGOES = {
    [EOBJECT0] = { scene = 2, flag = 1 },
    [EOBJECT1] = { scene = 3, flag = 2 },
}
CARGOES_NEEDED = 2

-- The progress pop-up carries ITEM0's icon, as retail sends it.
NOTICE_PROGRESS = 0
ITEM0_ICON = 21003

-- The scene after a search has nothing to show, so it plays without hiding the hotbar (as Sapphire does), which
-- would only make it flicker once the progress bar ends.
SEARCH_SCENE_FLAGS = 0

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
-- The cargo being searched, while its progress bar runs.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local cargo = CARGOES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and cargo ~= nil and not hasFlag(player, cargo.flag) then
        -- Searching shows a progress bar; the cargo's scene plays once it finishes (see onEventActionCast.)
        searching = cargo
        player:event_action(EVENT_ACTION_SEARCH)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if searching == nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, searching.scene, SEARCH_SCENE_FLAGS)
end

-- Walking into EVENTRANGE0 springs the ambush.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    player:spawn_quest_bnpc(ENEMY0)
    play(player, SCENE_AMBUSH, NO_DEFAULT_CAMERA | HIDE_HOTBAR)
end

-- One cargo searched.
function searched(player, flag)
    local count = getVar(player, 0, false) + 1 -- UI8AL
    setVar(player, 1, true, getVar(player, 1, true) + 1) -- UI8BH: another ITEM0
    player:event_notice(EVENT_ID, NOTICE_PROGRESS, { count, CARGOES_NEEDED, ITEM0_ICON })
    if count >= CARGOES_NEEDED then
        player:quest_sequence(EVENT_ID, 255)
        setVar(player, 0, false, 0)
        player:set_quest_var(EVENT_ID, 5, 0)
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

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif searching ~= nil and scene == searching.scene then
        local flag = searching.flag
        searching = nil
        searched(player, flag)
    elseif scene == SCENE_TRADE then
        -- 1 when the items were handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADE_OK then
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
