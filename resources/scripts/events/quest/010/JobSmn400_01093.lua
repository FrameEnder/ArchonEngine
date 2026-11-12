-- Quest: Shadowing the Summoner (the summoners' level 40 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest; its fixture is tools/questkit/fixtures/1093.json). Scene numbers
-- are the listing's in the order of the Quest sheet's listeners (battle NPCs none); the reward and the learning
-- cutscene follow JobWhm300_01079 (Bob's capture), as the recording's UI8A 0x11 then completion did.
-- Sequence 0:   ACTOR0 (Y'mhitra) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted).
-- Sequence 1:   ACTOR1 (the former Immortal Flame, in Southern Thanalan, scene 2, he walks off): notice 0 [1, 1],
--               sequence 255, and ENEMY0-2 (two hired fists and a hired bow, layer QB_JobSmn400_001 of wil_w1f4, as
--               retail spawned them 0.3 s after: position, level, HP and looks) come for you. Nothing waits on
--               them: Bob teleported away and they went. Scene 3 (a ScenarioMessage) wasn't seen played.
-- Sequence 255: ACTOR0's reward (scene 4, QuestReward): UI8AL and UI8AH 1, then the learning cutscene (scene 5,
--               LearningAction), and the quest completes.

AMBUSH = { ENEMY0, ENEMY1, ENEMY2 }

function ambushed(player, results)
    player:finish_event()
    player:event_notice(EVENT_ID, 0, { 1, 1 })
    player:quest_sequence(EVENT_ID, 255)
    for _, enemy in ipairs(AMBUSH) do
        player:spawn_quest_bnpc(enemy)
    end
    return true
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

-- A job's reward (JobWhm300_01079, Bob's Chronofoil capture 8d550e33): QuestReward taken, UI8AL and UI8AH go to 1
-- (0x11, retail's UpdateQuest as the next scene began) and the learning cutscene plays; the quest completes as it ends.
-- Use as a go_on on an entry whose scenes are {reward, cutscene}, with `learned` as its done.
LEARNING_FLAGS = 0xF8482EFB -- retail's, with parameter 0 (the same capture)
LEARNING_PARAMS = { 0 }
picked = 0
function reward_taken(player, index, results)
    if index ~= 1 or results[1] ~= 1 then
        return false
    end
    picked = results[2] or 0
    player:set_quest_var(EVENT_ID, 0, 0x11)
    return true
end

function learned(player, results)
    if scene_index < 2 then
        return -- the reward window was closed
    end
    player:finish_event()
    player:finish_quest(EVENT_ID, picked)
    return true
end

TALKS = {
    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered, done = accepted },
    { seq = 1, who = ACTOR1, scenes = { 2 }, done = ambushed },
    {
        seq = 255,
        who = ACTOR0,
        scenes = { 4, 5 },
        scene_flags = { [2] = LEARNING_FLAGS },
        scene_params = { [2] = LEARNING_PARAMS },
        go_on = reward_taken,
        done = learned,
    },
}
