-- Quest: Sinking Doesmaga (the Arcanists' level 30 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest with its quest battle won; its fixture is
-- tools/questkit/fixtures/461.json: sequences, variables and notices). Scene numbers are the listing's in the order of
-- the Quest sheet's listeners: the one each sequence is about has three (NpcTrade, a talk, an empty one), every
-- other event NPC one; flags the fork's usual ones.
-- Sequence 0:   ACTOR0 (Thubyrgeim) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted), handing
--               over ITEM0 (UI8BH 1, as retail's UpdateQuest had it from the start.)
-- Sequences 1-5: a chain of trades, each NPC taking what the last gave (NpcTrade, scene N: 1 when handed over) and
--               giving the next (scene N+1), with notice k and its item's icon: ACTOR1 (Rhylzirn, 2-3, notice 0
--               [29010]), ACTOR8 (Opylona, 11-12, 1 [24305]), ACTOR9 (Gegeruju, 20 and the cutscene 21, 2 [26001]),
--               ACTOR10 (Ealdgyth, 29-30, 3 [25919]: two items now, UI8BL 1 too), ACTOR11 (Drogo, 38-39, 4 [25057].)
--               Hirskskrat (ACTOR2) and K'lyhia (ACTOR5) are there throughout (5 and 8 in sequence 1, nine on in each.)
-- Sequence 6:   ACTOR2 (Hirskskrat) takes them (47, NpcTrade), asks (48, a YesNo either answer of which goes on) and
--               the cutscene (49): UI8BL cleared, notice 5, sequence 7. ACTOR5 (52) is there.
-- Sequence 7:   ACTOR12 (Hirskskrat): scene 55 (YesNoQuestBattle; 1 enters QUESTBATTLE0 (53) in TERRITORYTYPE0
--               (288).) Winning it moves to sequence 8 (content/questbattle/53.lua). ACTOR13 (K'lyhia, 56.)
-- Sequence 8:   ACTOR0 (scene 63): notice 7, sequence 255. ACTOR16 (the Yellowjacket, 64.)
-- Sequence 255: ACTOR0 takes ITEM0 back (scene 67, NpcTrade), then the reward (68, QuestReward, with the learning
--               cutscene in it.)

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, the fork's usual cutscene flags

-- Each trade gives this: notice, its icon (Bob's recording), the next sequence and UI8B (the items held.)
function trade(notice, icon, sequence, items)
    return function(player, results)
        if not all_played then
            return
        end
        player:set_quest_var(EVENT_ID, 1, items)
        player:event_notice(EVENT_ID, notice, icon and { icon } or {})
        player:quest_sequence(EVENT_ID, sequence)
    end
end

function accepted_with_item(player, results)
    if scene_index < 2 then
        return -- declined
    end
    player:accept_quest(EVENT_ID)
    player:set_quest_var(EVENT_ID, 1, 0x10) -- ITEM0 in UI8BH
    player:quest_sequence(EVENT_ID, 1)
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

-- A trade's go_on: NpcTrade returns 1 when the items were handed over (at the entry's scene `index`.)
function handed(index)
    return function(player, at, results)
        return at ~= index or results[1] == 1
    end
end

TALKS = {
    { seq = nil, who = ACTOR0, scenes = { 0, 1 }, go_on = offered, done = accepted_with_item },
    { seq = 1, who = ACTOR1, scenes = { 2, 3 }, go_on = handed(1), done = trade(0, 29010, 2, 0x10) },
    { seq = 1, who = ACTOR2, scenes = { 5 } },
    { seq = 1, who = ACTOR5, scenes = { 8 } },
    { seq = 2, who = ACTOR8, scenes = { 11, 12 }, go_on = handed(1), done = trade(1, 24305, 3, 0x10) },
    { seq = 2, who = ACTOR2, scenes = { 14 } },
    { seq = 2, who = ACTOR5, scenes = { 17 } },
    {
        seq = 3,
        who = ACTOR9,
        scenes = { 20, 21 },
        scene_flags = { [2] = CUTSCENE_FLAGS },
        go_on = handed(1),
        done = trade(2, 26001, 4, 0x10),
    },
    { seq = 3, who = ACTOR2, scenes = { 23 } },
    { seq = 3, who = ACTOR5, scenes = { 26 } },
    { seq = 4, who = ACTOR10, scenes = { 29, 30 }, go_on = handed(1), done = trade(3, 25919, 5, 0x11) },
    { seq = 4, who = ACTOR2, scenes = { 32 } },
    { seq = 4, who = ACTOR5, scenes = { 35 } },
    { seq = 5, who = ACTOR11, scenes = { 38, 39 }, go_on = handed(1), done = trade(4, 25057, 6, 0x11) },
    { seq = 5, who = ACTOR2, scenes = { 41 } },
    { seq = 5, who = ACTOR5, scenes = { 44 } },
    {
        seq = 6,
        who = ACTOR2,
        scenes = { 47, 48, 49 },
        scene_flags = { [3] = CUTSCENE_FLAGS },
        go_on = handed(1),
        done = trade(5, nil, 7, 0x10),
    },
    { seq = 6, who = ACTOR5, scenes = { 52 } },
    { seq = 7, who = ACTOR12, scenes = { 55 }, done = into_battle },
    { seq = 7, who = ACTOR13, scenes = { 56 } },
    { seq = 8, who = ACTOR0, scenes = { 63 }, done = moves_on(7, 255) },
    { seq = 8, who = ACTOR16, scenes = { 64 } },
    { seq = 255, who = ACTOR0, scenes = { 67, 68 }, flags = HIDE_HOTBAR, go_on = handed(1), done = reward },
}
