-- Quest: Tactical Planning (the Arcanists' level 10 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest with its quest battle won; its fixture is
-- tools/questkit/fixtures/456.json: sequences, variables and notices). The recording has no scenes: their numbers are
-- the listing's in the order of the Quest sheet's listeners (an event NPC one, an object two with its bar between,
-- the giver's offer two), which every quest Bob captured whole followed, and flags the fork's usual ones.
-- Sequence 0:   ACTOR0 (Thubyrgeim) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted).
-- Sequence 1:   three each of ENEMY0 (roselets, UI8AL, notice 0) and ENEMY1 (Redbelly jacks, UI8BH, notice 1), with
--               [count, 3]: sequence 2.
-- Sequence 2:   ACTOR0 (scene 2): notice 2, sequence 3.
-- Sequence 3:   ACTOR1 (K'lyhia, scene 3, she walks off): notice 3, sequence 4.
-- Sequence 4:   ACTOR2 (K'lyhia, scene 4: YesNoQuestBattle; 1 enters QUESTBATTLE0 (50) in TERRITORYTYPE0 (285).)
--               Winning it moves to sequence 5 (content/questbattle/50.lua). EOBJECT0 and EOBJECT1 (scenes 5-6, 7-8,
--               a search) and ACTOR3-5 (9, 10, 11) are there to look at.
-- Sequence 5:   ACTOR6 (K'lyhia, scene 12): notice 5, sequence 255. EOBJECT0 (13-14), ACTOR7-9 (15, 16, 17).
-- Sequence 255: ACTOR0's reward (scene 18, QuestReward). EOBJECT0 (19-20), ACTOR6 (21), ACTOR7-9 (22, 23, 24).

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001

ENEMIES = {
    [ENEMY0] = { notice = 0, byte = 0, high = false }, -- UI8AL
    [ENEMY1] = { notice = 1, byte = 1, high = true }, -- UI8BH
}
KILLS_SEQUENCE = 1

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

-- The talk under way (an entry of TALKS), which of its scenes is playing, and whether its bar is running.
talking = nil
scene_index = nil
in_bar = false
all_played = false

function play(player, scene, flags, params)
    player:play_scene(scene, flags, params or {})
end

-- A talk's scene: its own flags and parameters where the entry gives them (scene_flags, scene_params by index.)
function play_talk(player, talk, index)
    local flags = (talk.scene_flags and talk.scene_flags[index]) or talk.flags or HIDE_HOTBAR
    local params = talk.scene_params and talk.scene_params[index]
    play(player, talk.scenes[index], flags, params)
end

-- The first entry for this sequence and NPC (an emote's when `emote`), whose `when` (if any) says yes.
function start_talk(player, emote)
    local sequence = player:get_quest_sequence(EVENT_ID)
    for _, talk in ipairs(TALKS) do
        if talk.seq == sequence and talk.who == BASE_ID and (talk.emote or false) == emote
            and (talk.when == nil or talk.when(player)) then
            talking = talk
            scene_index = 1
            in_bar = false
            play_talk(player, talk, 1)
            return
        end
    end
    player:finish_event()
end

function onTalk(target, player)
    start_talk(player, false)
end

-- A talk's bar finished: its next scene.
function onEventActionCast(player)
    if talking == nil or not in_bar then
        player:finish_event()
        return
    end
    in_bar = false
    scene_index = scene_index + 1
    play_talk(player, talking, scene_index)
end

function sceneFinished(scene, results, player)
    if talking == nil or scene ~= talking.scenes[scene_index] then
        return
    end
    local talk = talking
    -- An object's bar comes after its first scene.
    if talk.bar ~= nil and scene_index == 1 then
        in_bar = true
        player:event_action(talk.bar)
        return
    end
    if scene_index < #talk.scenes and (talk.go_on == nil or talk.go_on(player, scene_index, results)) then
        scene_index = scene_index + 1
        play_talk(player, talk, scene_index)
        return
    end
    talking = nil
    -- Whether every scene played (a trade or offer turned down stops short.)
    all_played = scene_index == #talk.scenes
    -- scene_index stays the last scene's for `done` (an offer declined at its first.)
    if talk.done ~= nil and talk.done(player, results) then
        return
    end
    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end

-- What every quest's reward does: QuestReward returns 1 and the optional item picked.
function reward(player, results)
    if results[1] == 1 then
        player:finish_event()
        player:finish_quest(EVENT_ID, results[2] or 0)
        return true
    end
end

-- An offer: QuestOffer returns 1 when accepted, then the acceptance scene plays.
function offered(player, scene_index, results)
    return scene_index ~= 1 or results[1] == 1
end

function accepted(player, results)
    if scene_index < 2 then
        return -- declined
    end
    player:accept_quest(EVENT_ID)
    player:quest_sequence(EVENT_ID, 1)
end

-- YesNoQuestBattle returns 1 for yes: into QUESTBATTLE0.
function into_battle(player, results)
    if results[1] == 1 then
        player:finish_event()
        player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
        return true
    end
end

-- Moves on: a notice (if any) and the next sequence.
function moves_on(notice, sequence, args)
    return function(player, results)
        if notice ~= nil then
            player:event_notice(EVENT_ID, notice, args or {})
        end
        player:quest_sequence(EVENT_ID, sequence)
    end
end

-- Sequence 1's enemies: their todo (and notice), and where each count is kept (byte, high nibble); a kind at 3 counts
-- in UI8AH, and when both have, the next sequence with every variable cleared (the client's IsAcceptEvent stops
-- counting a kind at 3.)
KILLS_NEEDED = 3
function onBNpcKill(base_id, layout_id, player)
    local enemy = ENEMIES[base_id]
    if enemy == nil or player:get_quest_sequence(EVENT_ID) ~= KILLS_SEQUENCE then
        return
    end
    local count = getVar(player, enemy.byte, enemy.high)
    if count >= KILLS_NEEDED then
        return
    end
    count = count + 1
    player:event_notice(EVENT_ID, enemy.notice, { count, KILLS_NEEDED })
    if count < KILLS_NEEDED then
        setVar(player, enemy.byte, enemy.high, count)
        return
    end
    local kinds = 0
    for _ in pairs(ENEMIES) do
        kinds = kinds + 1
    end
    local done = getVar(player, 0, true) + 1 -- UI8AH
    if done >= kinds then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:set_quest_var(EVENT_ID, 1, 0)
        player:quest_sequence(EVENT_ID, KILLS_SEQUENCE + 1)
        return
    end
    setVar(player, enemy.byte, enemy.high, count)
    setVar(player, 0, true, done)
end

TALKS = {
    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered, done = accepted },
    { seq = 2, who = ACTOR0, scenes = { 2 }, done = moves_on(2, 3) },
    { seq = 3, who = ACTOR1, scenes = { 3 }, done = moves_on(3, 4) },
    { seq = 4, who = ACTOR2, scenes = { 4 }, done = into_battle },
    { seq = 4, who = EOBJECT0, scenes = { 5, 6 }, bar = EVENT_ACTION_SEARCH, flags = OBJECT_FLAGS },
    { seq = 4, who = EOBJECT1, scenes = { 7, 8 }, bar = EVENT_ACTION_SEARCH, flags = OBJECT_FLAGS },
    { seq = 4, who = ACTOR3, scenes = { 9 } },
    { seq = 4, who = ACTOR4, scenes = { 10 } },
    { seq = 4, who = ACTOR5, scenes = { 11 } },
    { seq = 5, who = ACTOR6, scenes = { 12 }, done = moves_on(5, 255) },
    { seq = 5, who = EOBJECT0, scenes = { 13, 14 }, bar = EVENT_ACTION_SEARCH, flags = OBJECT_FLAGS },
    { seq = 5, who = ACTOR7, scenes = { 15 } },
    { seq = 5, who = ACTOR8, scenes = { 16 } },
    { seq = 5, who = ACTOR9, scenes = { 17 } },
    { seq = 255, who = ACTOR0, scenes = { 18 }, done = reward },
    { seq = 255, who = EOBJECT0, scenes = { 19, 20 }, bar = EVENT_ACTION_SEARCH, flags = OBJECT_FLAGS },
    { seq = 255, who = ACTOR6, scenes = { 21 } },
    { seq = 255, who = ACTOR7, scenes = { 22 } },
    { seq = 255, who = ACTOR8, scenes = { 23 } },
    { seq = 255, who = ACTOR9, scenes = { 24 } },
}

