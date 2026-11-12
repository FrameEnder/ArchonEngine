-- Quest: Over the Rails (the Arcanists' level 15 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest with its quest battle won; its fixture is
-- tools/questkit/fixtures/458.json). Scene numbers are the listing's in the order of the Quest sheet's listeners (an
-- event NPC one, an object two with its bar between, battle NPCs none, the giver's offer two); the recording's field
-- fight caught one of them, the steersman's scene 12 with flags 0x2000, where that rule puts it.
-- Sequence 0:   ACTOR0 (Thubyrgeim) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted).
-- Sequence 1:   ACTOR1 (K'lyhia, scene 2): notice 0, sequence 2. ACTOR2 and ACTOR3 (3, 4) are there to talk to.
-- Sequence 2:   ACTOR4 (the steersman, scene 5): ENEMY0 and ENEMY1 (shifty-eyed vagrants, layer QB_ClsAcn150_001 of
--               sea_s1f4, as retail spawned them 0.2 s after the scene ended: position, level, HP and looks.) Each
--               down counts in UI8AL; both: notice 1, sequence 3, UI8AL cleared (retail's UpdateQuest 01, then 00 with
--               sequence 3.) Scene 5 itself tells the client's IsBattleNpcTriggerOwner: a talk while they're out.
--               ACTOR1-3 (7, 8, 9) and EOBJECT0 (10-11, empty) are there.
-- Sequence 3:   ACTOR4 (scene 12): notice 2, sequence 4. ACTOR1-3 (13, 14, 15), EOBJECT0 (16-17.)
-- Sequence 4:   EOBJECT1 (the landing boat): scene 18, EVENT_ACTION_WAITING2_MIDDLE's bar (the sheet's), scene 19
--               (YesNoQuestBattle; 1 enters QUESTBATTLE0 (51) in TERRITORYTYPE0 (286).) Winning it moves to sequence
--               255 (content/questbattle/51.lua). EOBJECT0 (21-22), ACTOR1-3 (23, 24, 25.) Scene 20 (a cutscene with
--               K'lyhia) wasn't seen played: retail's battle ended on its own scene 5 and the next quest change was
--               the reward.
-- Sequence 255: ACTOR0's reward (scene 26, QuestReward). ACTOR5-7 (27, 28, 29), EOBJECT0 (30-31.)

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
VAGRANTS = { ENEMY0, ENEMY1 }


-- The steersman's talk brings them out (ones still out aren't brought out again.)
function vagrants_come(player, results)
    player:finish_event()
    for _, vagrant in ipairs(VAGRANTS) do
        player:spawn_quest_bnpc(vagrant)
    end
    return true
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 or (layout_id ~= ENEMY0 and layout_id ~= ENEMY1) then
        return
    end
    local count = getVar(player, 0, false) + 1
    if count >= #VAGRANTS then
        setVar(player, 0, false, 0)
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    else
        setVar(player, 0, false, count)
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

TALKS = {
    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered, done = accepted },
    { seq = 1, who = ACTOR1, scenes = { 2 }, done = moves_on(0, 2) },
    { seq = 1, who = ACTOR2, scenes = { 3 } },
    { seq = 1, who = ACTOR3, scenes = { 4 } },
    { seq = 2, who = ACTOR4, scenes = { 5 }, done = vagrants_come },
    { seq = 2, who = ACTOR1, scenes = { 7 } },
    { seq = 2, who = ACTOR2, scenes = { 8 } },
    { seq = 2, who = ACTOR3, scenes = { 9 } },
    { seq = 2, who = EOBJECT0, scenes = { 10, 11 }, flags = OBJECT_FLAGS },
    { seq = 3, who = ACTOR4, scenes = { 12 }, done = moves_on(2, 4) },
    { seq = 3, who = ACTOR1, scenes = { 13 } },
    { seq = 3, who = ACTOR2, scenes = { 14 } },
    { seq = 3, who = ACTOR3, scenes = { 15 } },
    { seq = 3, who = EOBJECT0, scenes = { 16, 17 }, flags = OBJECT_FLAGS },
    {
        seq = 4,
        who = EOBJECT1,
        scenes = { 18, 19 },
        bar = EVENT_ACTION_WAITING2_MIDDLE,
        flags = OBJECT_FLAGS,
        done = into_battle,
    },
    { seq = 4, who = EOBJECT0, scenes = { 21, 22 }, flags = OBJECT_FLAGS },
    { seq = 4, who = ACTOR1, scenes = { 23 } },
    { seq = 4, who = ACTOR2, scenes = { 24 } },
    { seq = 4, who = ACTOR3, scenes = { 25 } },
    { seq = 255, who = ACTOR0, scenes = { 26 }, done = reward },
    { seq = 255, who = ACTOR5, scenes = { 27 } },
    { seq = 255, who = ACTOR6, scenes = { 28 } },
    { seq = 255, who = ACTOR7, scenes = { 29 } },
    { seq = 255, who = EOBJECT0, scenes = { 30, 31 }, flags = OBJECT_FLAGS },
}
