-- Quest 111 (SubSea001): offered in the Limsa Lominsa opening once Coming to Limsa Lominsa (107) is done, alongside
-- Close to Home; its ferry is the opening's way out into the real Lower Decks.
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- opening on retail (2026-10-20):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts), then the client's empty scene 1; when it returns,
--               sequence 1 (retail sent no AcceptQuest for this one, only the journal update.)
-- Sequence 1:   ACTOR1 (todo 0, at the opening's exit) asks (scene 2): the client only offers the yes/no once Close to
--               Home (QUEST0 or QUEST1) is accepted, and 1 means go: sequence 2, and the player goes through a loading
--               screen into TERRITORYTYPE0 (the Lower Decks) at POPRANGE0. Asked again in sequence 2 it's scene 15.
-- Sequence 2:   pick the six herbs (EOBJECT0..5, todo 1, any order): each is scene 2n + 3 for EOBJECTn, then
--               EVENT_ACTION_PROCESS (the progress bar), then scene 2n + 4, all with HIDE_HOTBAR | NO_DEFAULT_CAMERA;
--               then notice 1 with [count, 6, 25002]. UI8AL counts them (the todo), UI8BH is ITEM0 held (the herbs,
--               GetEventItems), BitFlag8 n + 1 marks EOBJECTn. The sixth clears all but UI8BH (6) and moves to 255.
-- Sequence 255: ACTOR2: the herbs handed over (scene 16, an NpcTrade: 1 if given), then the reward (scene 17, 1
--               completes it.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1 -- empty in the client's script
SCENE_FERRY = 2
SCENE_FERRY_AGAIN = 15
SCENE_TRADE = 16
SCENE_REWARD = 17

HERBS = { EOBJECT0, EOBJECT1, EOBJECT2, EOBJECT3, EOBJECT4, EOBJECT5 }

NOTICE_HERB = 1
HERB_ICON = 25002 -- retail's third notice argument (an icon, as in the other quests' item notices)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001, as retail played the herbs' scenes

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

function hasBitFlag8(player, n)
    return (player:get_quest_var(EVENT_ID, 5) & (1 << (8 - n))) ~= 0
end

function setBitFlag8(player, n)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (1 << (8 - n)))
end

-- Which herb BASE_ID is (0..5), if it's one.
function herbIndex()
    for i, id in ipairs(HERBS) do
        if id == BASE_ID then
            return i - 1
        end
    end
    return nil
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The herb this event is picking (a new Lua state per event.)
picking = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local herb = herbIndex()

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_FERRY, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_FERRY_AGAIN, HIDE_HOTBAR)
    elseif sequence == 2 and herb ~= nil and getVar(player, 0, false) < #HERBS and not hasBitFlag8(player, herb + 1) then
        picking = herb
        play(player, 2 * herb + 3, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if picking == nil or player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, 2 * picking + 4, OBJECT_FLAGS)
end

function herbPicked(player, herb)
    local count = getVar(player, 0, false) + 1
    if count >= #HERBS then
        for b = 0, 5 do
            player:set_quest_var(EVENT_ID, b, 0)
        end
        setVar(player, 1, true, count) -- retail keeps UI8BH (the herbs held) in sequence 255
        player:event_notice(EVENT_ID, NOTICE_HERB, { count, #HERBS, HERB_ICON })
        player:quest_sequence(EVENT_ID, 255)
    else
        setVar(player, 0, false, count)
        setVar(player, 1, true, count)
        setBitFlag8(player, herb + 1)
        player:event_notice(EVENT_ID, NOTICE_HERB, { count, #HERBS, HERB_ICON })
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
    elseif scene == SCENE_FERRY or scene == SCENE_FERRY_AGAIN then
        if results[1] == 1 then
            if scene == SCENE_FERRY then
                player:quest_sequence(EVENT_ID, 2)
            end
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
    elseif picking ~= nil and scene == 2 * picking + 3 then
        -- Picking shows a progress bar; the herb's second scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_PROCESS)
        return
    elseif picking ~= nil and scene == 2 * picking + 4 then
        herbPicked(player, picking)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
