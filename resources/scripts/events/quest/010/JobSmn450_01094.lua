-- Quest: Allagan Attire (the summoners' level 45 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest; its fixture is tools/questkit/fixtures/1094.json). Its field fights
-- caught scenes 3, 5, 6 and 8 (flags 0x2000), where the order of the Quest sheet's listeners puts them (battle NPCs
-- none); the rest are the listing's by that order.
-- Sequence 0:   ACTOR0 (Y'mhitra) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted).
-- Sequence 1:   ACTOR1 (Rammbroes, in Mor Dhona, scene 2, a cutscene): notice 0, sequence 2.
-- Sequences 2, 4, 6: each researcher in turn (ACTOR2, ACTOR3, ACTOR4: scenes 3, 5, 7) and an Allagan mirrorknight
--               comes out 0.4-0.6 s after (ENEMY0, ENEMY1, ENEMY2: layer QB_JobSmn450_001 of lak_l1f1, retail's
--               positions, level and HP); its fall gives notice 1, 3, 5 and the next sequence. The scene itself asks
--               the client's IsBattleNpcTriggerOwner: a talk while it's out.
-- Sequences 3, 5, 7: the researcher again (scenes 4, 6, 8): notice 2, 4, 6 with its item's icon (45333, 44191,
--               46351, Bob's recording), then retail's EventLogMessage LOGMESSAGE0 (3677) from the quest, and the next
--               sequence.
-- Sequence 8:   ACTOR1 (scene 9): notice 7 [41172], LOGMESSAGE0, sequence 255.
-- Sequence 255: ACTOR0's reward (scene 10, QuestReward.)

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, the fork's usual cutscene flags

-- Each researcher's mirrorknight, the notice its fall gives and the sequence after.
KNIGHTS = {
    [2] = { knight = ENEMY0, notice = 1, next = 3 },
    [4] = { knight = ENEMY1, notice = 3, next = 5 },
    [6] = { knight = ENEMY2, notice = 5, next = 7 },
}

function knight_comes(player, results)
    player:finish_event()
    player:spawn_quest_bnpc(KNIGHTS[player:get_quest_sequence(EVENT_ID)].knight)
    return true
end

function onBNpcKill(base_id, layout_id, player)
    local step = KNIGHTS[player:get_quest_sequence(EVENT_ID)]
    if step == nil or layout_id ~= step.knight then
        return
    end
    player:event_notice(EVENT_ID, step.notice, {})
    player:quest_sequence(EVENT_ID, step.next)
end

-- A researcher's (or Rammbroes's) thanks: the notice with its icon, LOGMESSAGE0, the next sequence.
function thanked(notice, icon, sequence)
    return function(player, results)
        player:event_notice(EVENT_ID, notice, { icon })
        player:event_log_message(EVENT_ID, LOGMESSAGE0, {})
        player:quest_sequence(EVENT_ID, sequence)
    end
end

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

-- What every quest's reward does: QuestReward returns 1 and the optional item picked. Closed instead, the quest stays
-- open with UI8AL and UI8AH 1 (0x11), as retail sets it (414, 521, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11
function reward(player, results)
    if results[1] == 1 then
        player:finish_event()
        player:finish_quest(EVENT_ID, results[2] or 0)
        return true
    end
    if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
        player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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

-- Moves on: a notice (if any) and the next sequence.
function moves_on(notice, sequence, args)
    return function(player, results)
        if notice ~= nil then
            player:event_notice(EVENT_ID, notice, args or {})
        end
        player:quest_sequence(EVENT_ID, sequence)
    end
end

TALKS = {
    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered, done = accepted },
    { seq = 1, who = ACTOR1, scenes = { 2 }, flags = CUTSCENE_FLAGS, done = moves_on(0, 2) },
    { seq = 2, who = ACTOR2, scenes = { 3 }, done = knight_comes },
    { seq = 3, who = ACTOR2, scenes = { 4 }, done = thanked(2, 45333, 4) },
    { seq = 4, who = ACTOR3, scenes = { 5 }, done = knight_comes },
    { seq = 5, who = ACTOR3, scenes = { 6 }, done = thanked(4, 44191, 6) },
    { seq = 6, who = ACTOR4, scenes = { 7 }, done = knight_comes },
    { seq = 7, who = ACTOR4, scenes = { 8 }, done = thanked(6, 46351, 8) },
    { seq = 8, who = ACTOR1, scenes = { 9 }, done = thanked(7, 41172, 255) },
    { seq = 255, who = ACTOR0, scenes = { 10 }, done = reward },
}
