-- Quest: Spirithold Broken (Gridania)
--
-- Checked against the client's quest script (kawari-questdump), Sapphire's script and a live recording of the quest:
-- Sequence 0:   ACTOR0 offers the quest (scene 0, returns 1 when accepted), then explains it in a cutscene (scene 1.)
-- Sequence 1:   report to ACTOR1 at Spirithold (scene 2.)
-- Sequence 2:   find the five who were left behind, ACTOR2 to ACTOR6, in any order: a scene (3, 6, 9, 12, 15), aiding
--               them (EVENT_ACTION_, "Aiding", a progress bar), then their scene (4, 7, 10, 13 and 16.) UI8AL counts them for the todo and
--               each sets its BitFlag8 (1 to 5, in UI8F), which stops the client offering them again. With all five
--               found, retail clears the variables and moves to sequence 3. ACTOR7 (the Hearer) only has a line (18.)
-- Sequence 3:   talk to the Hearer (ACTOR7) again, or EOBJECT0 next to him, and agree to enter the quest battle
--               (scene 19, YesNoQuestBattle returns 1.) That it's the Hearer is Sapphire's, and what the client sends.
--               QUESTBATTLE0 takes place in TERRITORYTYPE0, and winning it moves the quest to sequence 4.
-- Sequence 4:   coming back into the Central Shroud plays the cutscenes after it (scene 23), then sequence 5.
-- Sequence 5:   report to ACTOR1 (scene 24.)
-- Sequence 255: Mother Miounne (ACTOR8) and the reward (scene 25.)
-- Notices (retail): 0 on reporting to ACTOR1, 1 [count, 5] for each one found, 4 on reporting to ACTOR1 after the
-- battle. None for the battle or the cutscenes after it.

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
    [ACTOR2] = { before = 3, scene = 4, flag = 1 },
    [ACTOR3] = { before = 6, scene = 7, flag = 2 },
    [ACTOR4] = { before = 9, scene = 10, flag = 3 },
    [ACTOR5] = { before = 12, scene = 13, flag = 4 },
    [ACTOR6] = { before = 15, scene = 16, flag = 5 },
}
EVACUEES_NEEDED = 5

-- The notices retail sends (Bob's Chronofoil capture of 2026-10-14.)
NOTICE_REPORTED = 0
NOTICE_FOUND = 1
NOTICE_REPORTED_AFTER = 4

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays its cutscenes (Bob's Chronofoil capture of 2026-10-14)
-- Aiding someone, as retail plays it (Bob's Chronofoil capture of 2026-10-14): a scene before the progress bar
-- (HIDE_HOTBAR | NO_DEFAULT_CAMERA), and their scene after it with INVINCIBLE too.
AID_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
AIDED_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE
AFTERMATH_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, as retail plays the aftermath (Bob's Chronofoil capture of 2026-10-14)
REWARD_FLAGS = SET_BASE | HIDE_HOTBAR | DISABLE_SKIP -- 0xF8482EFB, retail's (Bob's Chronofoil capture of 2026-10-14)

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
        -- Their first scene, then the progress bar (see sceneFinished), then their scene once it finishes.
        aiding = evacuee
        play(player, evacuee.before, AID_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR7 then
        play(player, SCENE_HEARER, HIDE_HOTBAR)
    elseif sequence == 3 and (BASE_ID == ACTOR7 or BASE_ID == EOBJECT0) then
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
    play(player, aiding.scene, AIDED_FLAGS)
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
    player:event_notice(EVENT_ID, NOTICE_FOUND, { count, EVACUEES_NEEDED })
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
        player:event_notice(EVENT_ID, NOTICE_REPORTED)
        player:quest_sequence(EVENT_ID, 2)
    elseif aiding ~= nil and scene == aiding.before then
        -- Aiding shows a progress bar; their scene plays once it finishes (see onEventActionCast.)
        player:event_action(EVENT_ACTION_)
        return
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
        player:event_notice(EVENT_ID, NOTICE_REPORTED_AFTER)
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
