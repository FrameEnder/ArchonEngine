-- Quest: Spirithold Broken (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ACTOR0 offers the quest (scene 0, returns 1 when accepted), then explains it in a cutscene (scene 1.)
-- Sequence 1:   report to ACTOR1 at Spirithold (scene 2.)
-- Sequence 2:   find the five who were left behind, ACTOR2 to ACTOR6, in any order: aid each one (EVENT_ACTION_,
--               "Aiding", a progress bar), then their scene (4, 7, 10, 13 and 16.) UI8AL counts them for the todo and
--               each sets its BitFlag8 (1 to 5, in UI8F), which stops the client offering them again. With all five
--               found, retail clears the variables and moves to sequence 3. ACTOR7 (the Hearer) only has a line (18.)
-- Sequence 3:   go deeper, EOBJECT0, and agree to enter the quest battle (scene 19, YesNoQuestBattle returns 1.)
--               QUESTBATTLE0 takes place in TERRITORYTYPE0, and winning it moves the quest to sequence 4.
-- Sequence 4:   coming back into the Central Shroud plays the cutscenes after it (scene 23), then sequence 5.
-- Sequence 5:   report to ACTOR1 (scene 24.)
-- Sequence 255: Mother Miounne (ACTOR8) and the reward (scene 25.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ALESTAN = 2
SCENE_HEARER = 18
SCENE_ENTER_BATTLE = 19
SCENE_AFTERMATH = 23
SCENE_ALESTAN_AFTER = 24
SCENE_FINISH = 25

-- The five to find: their scene and BitFlag8 index.
EVACUEES = {
    [ACTOR2] = { scene = 4, flag = 1 },
    [ACTOR3] = { scene = 7, flag = 2 },
    [ACTOR4] = { scene = 10, flag = 3 },
    [ACTOR5] = { scene = 13, flag = 4 },
    [ACTOR6] = { scene = 16, flag = 5 },
}
EVACUEES_NEEDED = 5

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

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Who's being aided, while the progress bar runs.
aiding = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local evacuee = EVACUEES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ALESTAN, HIDE_HOTBAR)
    elseif sequence == 2 and evacuee ~= nil and not hasFlag(player, evacuee.flag) then
        -- Aiding shows a progress bar; their scene plays once it finishes (see onEventActionCast.)
        aiding = evacuee
        player:event_action(EVENT_ACTION_)
    elseif sequence == 2 and BASE_ID == ACTOR7 then
        play(player, SCENE_HEARER, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == EOBJECT0 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR1 then
        play(player, SCENE_ALESTAN_AFTER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR8 then
        play(player, SCENE_FINISH, REWARD_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if aiding == nil or player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    play(player, aiding.scene, HIDE_HOTBAR)
end

-- Back in the Central Shroud after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 4 then
        play(player, SCENE_AFTERMATH, AFTERMATH_FLAGS)
    else
        player:finish_event()
    end
end

-- One of the five found. Once all are, retail clears the variables and moves on.
function found(player, flag)
    local count = getVar(player, 0, false) + 1 -- UI8AL
    setVar(player, 0, false, count)
    setFlag(player, flag)
    if count >= EVACUEES_NEEDED then
        player:quest_sequence(EVENT_ID, 3)
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
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
    elseif scene == SCENE_ALESTAN then
        player:quest_sequence(EVENT_ID, 2)
    elseif aiding ~= nil and scene == aiding.scene then
        local flag = aiding.flag
        aiding = nil
        found(player, flag)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_ALESTAN_AFTER then
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
