-- Quest: Chasing Shadows (Gridania)
--
-- Checked against the client's quest script (kawari-questdump):
-- Sequence 0:   ACTOR0 offers the quest (scene 0, returns 1 when accepted), then explains it in a cutscene (scene 1.)
-- Sequence 1:   examine the magicked blade, EOBJECT0 (scene 2, then EVENT_ACTION_PROCESS_SHOR), and agree to enter the
--               quest battle (scene 3, YesNoQuestBattle returns 1.) Its todo is checked by UI8AL.
--               QUESTBATTLE0 takes place in TERRITORYTYPE0, and winning it moves the quest to sequence 2.
-- Sequence 2:   coming back into the Central Shroud plays the aftermath cutscenes (scene 5.)
-- Sequence 3:   examine the second magicked blade, EOBJECT1 (EVENT_ACTION_PROCESS_SHOR), which gives you ITEM0.
--               ITEM0 is held in UI8BH from here on.
-- Sequence 255: hand ITEM0 to ACTOR0 (trade scene 9, returns 1 when handed over), then the reward (scene 10.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_EXAMINE_BLADE = 2
SCENE_ENTER_BATTLE = 3
SCENE_AFTERMATH = 5
SCENE_TRADE = 9
SCENE_FINISH = 10

CUTSCENE_FLAGS = FADE_OUT | CONDITION_CUTSCENE | HIDE_UI
AFTERMATH_FLAGS = NO_DEFAULT_CAMERA | CONDITION_CUTSCENE | SILENT_ENTER_TERRI_ENV | HIDE_HOTBAR | SILENT_ENTER_TERRI_BGM | SILENT_ENTER_TERRI_SE | DISABLE_STEALTH | UNK1 | LOCK_HUD | LOCK_HOTBAR | DISABLE_CANCEL_EMOTE
REWARD_FLAGS = FADE_OUT | HIDE_HOTBAR | CONDITION_CUTSCENE | HIDE_UI

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

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        if getVar(player, 0, false) == 1 then
            -- Already examined, they only need to agree to the battle this time.
            play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
        else
            play(player, SCENE_EXAMINE_BLADE, HIDE_HOTBAR)
        end
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        player:event_action(EVENT_ACTION_PROCESS_SHOR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Back in the Central Shroud after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_AFTERMATH, AFTERMATH_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == 1 then
        setVar(player, 0, false, 1) -- UI8AL: the blade was examined
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 3 then
        setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
        player:finish_event()
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_EXAMINE_BLADE then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_PROCESS_SHOR)
        return
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_FINISH, REWARD_FLAGS)
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
