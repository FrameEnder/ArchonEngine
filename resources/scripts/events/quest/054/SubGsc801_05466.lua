-- Quest: Keyward Bound (a Gold Saucer side quest, the way into Keybound Square)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest, played 2026-11-04; its fixture is tools/questkit/fixtures/5466.json:
-- sequences and variables; the export's notices from that day were read with the wrong version's opcodes.) Scene
-- numbers are the listing's in the order of the Quest sheet's listeners (W'zazanh's YesNo in each sequence, the zone-in
-- listener 5020000's cutscene); flags the fork's usual ones.
-- Sequence 0:   ACTOR0 (the catlike mammet) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted,
--               it walks off.)
-- Sequence 1:   ACTOR1 (W'zazanh, scene 2: a YesNo, yes fades out): sequence 2 (retail's, still in the Gold Saucer)
--               and the player is taken to TERRITORYTYPE0 (1360, Keybound Square) at POPRANGE0.
-- Sequence 2:   entering TERRITORYTYPE0: scene 3 (the cutscene CUT_SCENE_01), sequence 255. ACTOR1 (scene 4, the
--               same YesNo) takes you there again.
-- Sequence 255: ACTOR2 (the magitype master)'s reward (scene 5, QuestReward). ACTOR1 (scene 6) takes you there.

-- W'zazanh's YesNo: yes (1) takes you to Keybound Square.
function to_keybound_square(sequence)
    return function(player, results)
        if results[1] ~= 1 then
            return
        end
        if sequence ~= nil then
            player:quest_sequence(EVENT_ID, sequence)
        end
        player:finish_event()
        player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
        return true
    end
end

function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) ~= 2 then
        player:finish_event()
        return
    end
    talking = ENTERED
    scene_index = 1
    in_bar = false
    play_talk(player, ENTERED, 1)
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
    { seq = 1, who = ACTOR1, scenes = { 2 }, done = to_keybound_square(2) },
    { seq = 2, who = ACTOR1, scenes = { 4 }, done = to_keybound_square(nil) },
    { seq = 255, who = ACTOR2, scenes = { 5 }, done = reward },
    { seq = 255, who = ACTOR1, scenes = { 6 }, done = to_keybound_square(nil) },
}

-- The zone-in's cutscene, not anyone's talk.
ENTERED = { seq = 2, scenes = { 3 }, flags = INITIAL_CUTSCENE_FLAGS, done = moves_on(nil, 255) }
