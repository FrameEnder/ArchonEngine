-- Quest: Grimoire Fandango (the Arcanists' level 25 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest; its fixture is tools/questkit/fixtures/460.json: sequences,
-- variables, notices, and the EventFinish of each step, whose type says which were emotes: type 2 with emote 11 on
-- the box, Memeroon and K'lyhia, the Quest sheet's listeners of type 2.) The field fight with the slugs caught the
-- box's scenes 22 and 23 (0x2001, 2.3 s apart: its bar between.) The other scene numbers are read from the listing:
-- the listeners' order (an event NPC one, an object two with its bar between) where it fits, what the scene does
-- where it doesn't (this quest's emote listeners and boxes don't follow the order cleanly); the empty scenes of
-- EOBJECT4 (the other box) and of K'lyhia's talk in sequence 9 (16-19, 21, 24-25, 29-30) aren't played.
-- Sequence 0:   ACTOR0 (Thubyrgeim) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted).
-- Sequence 1:   ACTOR1 (the witness, scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (Tanga Tonga, scene 3): notice 1, sequence 3. ACTOR1 (4) is there.
-- Sequence 3:   the three hidden parchments, EOBJECT0-2 (scenes 5-6, 7-8, 9-10 with EVENT_ACTION_SEARCH_MIDDLE's bar
--               between): each counts in UI8AH, marks its own nibble (EOBJECT0 UI8AL, EOBJECT1 UI8BH, EOBJECT2 UI8BL,
--               as the client's IsAcceptEvent reads them) and BitFlag8 flag (1-3), with ITEM0's count in UI8CH, and
--               notice 2 [count, 3, 25906]; the third clears them all for sequence 4 with ITEM0 (3) in UI8BH. ACTOR2
--               (11) is there.
-- Sequence 4:   ACTOR2 takes them (scene 12, NpcTrade: 1 when handed over, then 13): UI8BH cleared, notice 3,
--               sequence 5.
-- Sequence 5:   EOBJECT3 (the old wooden box): a talk shows scene 14; an emote, scene 15: notice 4, sequence 6.
-- Sequence 6:   EOBJECT3: scene 20, EVENT_ACTION_SEARCH's bar, scene 21; ENEMY0-2 (tree slugs, layer QB_ClsAcn250_001
--               of sea_s1f5, out as the talk ended), notice 5, sequence 7.
-- Sequence 7:   EOBJECT3: scene 22, the bar, scene 23 (the memo): notice 6, sequence 8.
-- Sequence 8:   ACTOR3 (Memeroon): a talk, scene 26; an emote, scenes 27 and 28 (the cutscene CUT_CLSACN25010):
--               ITEM1 in UI8BH, notice 7 [26002], sequence 9.
-- Sequence 9:   ACTOR4 (K'lyhia), an emote: scene 31, notice 8, sequence 10.
-- Sequence 10:  ACTOR4: scene 32 (a YesNo, either answer goes on), 33 (NpcTrade of ITEM1), 34 (a cutscene): UI8BH
--               cleared, notice 9, sequence 255.
-- Sequence 255: ACTOR0's reward (scene 35, QuestReward). ACTOR4 (36) is there.

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, the fork's usual cutscene flags
PARCHMENT_ICON = 25906 -- notice 2's third argument (Bob's recording)
ITEM1_ICON = 26002 -- notice 7's argument (the same)
PARCHMENTS_NEEDED = 3
-- Each parchment's own nibble and BitFlag8 flag (retail's UpdateQuests: EOBJECT0 0x11 .. 0x80, EOBJECT2 .. 0x01 ..
-- 0xA0.)
PARCHMENTS = {
    [EOBJECT0] = { byte = 0, high = false, flag = 1 },
    [EOBJECT1] = { byte = 1, high = true, flag = 2 },
    [EOBJECT2] = { byte = 1, high = false, flag = 3 },
}
SLUGS = { ENEMY0, ENEMY1, ENEMY2 }

function not_found(player)
    local parchment = PARCHMENTS[BASE_ID]
    return player:get_quest_var(EVENT_ID, 5) & (0x80 >> (parchment.flag - 1)) == 0
end

function parchment_found(player, results)
    local parchment = PARCHMENTS[BASE_ID]
    local count = getVar(player, 0, true) + 1
    player:event_notice(EVENT_ID, 2, { count, PARCHMENTS_NEEDED, PARCHMENT_ICON })
    if count >= PARCHMENTS_NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        setVar(player, 1, true, count) -- ITEM0 in UI8BH
        player:quest_sequence(EVENT_ID, 4)
        return
    end
    setVar(player, 0, true, count)
    setVar(player, parchment.byte, parchment.high, 1)
    setVar(player, 2, true, count) -- ITEM0 in UI8CH
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (parchment.flag - 1)))
end

function handed_over(notice, sequence)
    return function(player, results)
        if not all_played then
            return
        end
        setVar(player, 1, true, 0)
        player:event_notice(EVENT_ID, notice, {})
        player:quest_sequence(EVENT_ID, sequence)
    end
end

function slugs_come(player, results)
    player:finish_event()
    for _, slug in ipairs(SLUGS) do
        player:spawn_quest_bnpc(slug)
    end
    player:event_notice(EVENT_ID, 5, {})
    player:quest_sequence(EVENT_ID, 7)
    return true
end

function memo_given(player, results)
    setVar(player, 1, true, 1) -- ITEM1 in UI8BH
    player:event_notice(EVENT_ID, 7, { ITEM1_ICON })
    player:quest_sequence(EVENT_ID, 9)
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

-- An emote on an NPC the quest listens to for one (a listener of type 2): any emote does.
function onEmote(emote, target, player)
    start_talk(player, true)
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

-- A trade's go_on: NpcTrade returns 1 when the items were handed over (at the entry's scene `index`.)
function handed(index)
    return function(player, at, results)
        return at ~= index or results[1] == 1
    end
end

TALKS = {
    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered, done = accepted },
    { seq = 1, who = ACTOR1, scenes = { 2 }, done = moves_on(0, 2) },
    { seq = 2, who = ACTOR2, scenes = { 3 }, done = moves_on(1, 3) },
    { seq = 2, who = ACTOR1, scenes = { 4 } },
    {
        seq = 3,
        who = EOBJECT0,
        scenes = { 5, 6 },
        bar = EVENT_ACTION_SEARCH_MIDDLE,
        flags = OBJECT_FLAGS,
        when = not_found,
        done = parchment_found,
    },
    {
        seq = 3,
        who = EOBJECT1,
        scenes = { 7, 8 },
        bar = EVENT_ACTION_SEARCH_MIDDLE,
        flags = OBJECT_FLAGS,
        when = not_found,
        done = parchment_found,
    },
    {
        seq = 3,
        who = EOBJECT2,
        scenes = { 9, 10 },
        bar = EVENT_ACTION_SEARCH_MIDDLE,
        flags = OBJECT_FLAGS,
        when = not_found,
        done = parchment_found,
    },
    { seq = 3, who = ACTOR2, scenes = { 11 } },
    { seq = 4, who = ACTOR2, scenes = { 12, 13 }, go_on = handed(1), done = handed_over(3, 5) },
    { seq = 5, who = EOBJECT3, scenes = { 14 }, flags = OBJECT_FLAGS },
    { seq = 5, who = EOBJECT3, emote = true, scenes = { 15 }, flags = OBJECT_FLAGS, done = moves_on(4, 6) },
    { seq = 6, who = EOBJECT3, scenes = { 20, 21 }, bar = EVENT_ACTION_SEARCH, flags = OBJECT_FLAGS, done = slugs_come },
    { seq = 7, who = EOBJECT3, scenes = { 22, 23 }, bar = EVENT_ACTION_SEARCH, flags = OBJECT_FLAGS, done = moves_on(6, 8) },
    { seq = 8, who = ACTOR3, scenes = { 26 } },
    {
        seq = 8,
        who = ACTOR3,
        emote = true,
        scenes = { 27, 28 },
        scene_flags = { [2] = CUTSCENE_FLAGS },
        done = memo_given,
    },
    { seq = 9, who = ACTOR4, emote = true, scenes = { 31 }, done = moves_on(8, 10) },
    {
        seq = 10,
        who = ACTOR4,
        scenes = { 32, 33, 34 },
        scene_flags = { [3] = CUTSCENE_FLAGS },
        go_on = handed(2),
        done = handed_over(9, 255),
    },
    { seq = 255, who = ACTOR0, scenes = { 35 }, done = reward },
    { seq = 255, who = ACTOR4, scenes = { 36 } },
}
