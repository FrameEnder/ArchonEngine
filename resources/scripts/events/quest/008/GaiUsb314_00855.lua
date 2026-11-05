-- Quest: In the Company of Heroes
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b4ce5e3f (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/855.json), d65126c9
-- (an Arcanist, `855-d65126c9.json`) and 811eb9ff (a White Mage, `855-811eb9ff.json`):
-- Sequence 0:   ACTOR0 (Y'shtola) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, A Final Ignominy (854), is in the journal), scene 1, then the cutscene
--               (scene 2, 0xF8402EFB), where the client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (the bronze-skinned beauty, scene 3; offered while UI8AL < 1 and BitFlag16 1 unset): notice 0,
--               sequence 2.
-- Sequence 2:   EOBJECT8 (the exotic feast, offered on the same terms): scene 26 (0x2001), then the cutscene (scene
--               27, 0xF8406EF3): notice 1, sequence 3.
-- Sequence 3:   the five of them, in any order, each once: ACTOR8 (Wheiskaet, scenes 51 and 52), ACTOR5 (Landenel,
--               53), ACTOR4 (U'odh Nunh, 55), ACTOR6 (Brayflox, 57), ACTOR7 (Shamani Lohmani, 59 and 60). Each sets its
--               own nibble (UI8AL, UI8BH, UI8BL, UI8CH, UI8CL) and BitFlag16 (1-5, flag 1 being UI8E's top bit, as
--               retail set them), UI8AH (the count, the todo's) one more, notice 2 [count, 5]; the fifth: the variables
--               cleared, sequence 255. Talked to again, Landenel, U'odh Nunh, Brayflox and Shamani say their line for
--               after (54, 56, 58, 61); Wheiskaet isn't offered again.
-- Sequence 255: ACTOR0 rewards it (scene 75, a cutscene and a yield: QuestReward, 1.)
-- Everyone else the client offers has a line for each sequence (in the client's IsAcceptEvent order, which its scenes
-- follow: e.g. sequence 1's ACTOR2-7 are scenes 4-9, its EOBJECT0-7 two scenes each from 10.) Retail was seen playing
-- only the ones above; the feasting tables' and the unnamed NPCs' scenes are empty in the client's script.

SCENE_OFFER = 0
SCENE_OFFER_TALK = 1
SCENE_ACCEPTED = 2
SCENE_BEAUTY = 3
SCENE_FEAST = 26
SCENE_FEAST_CUTSCENE = 27
SCENE_REWARD = 75
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
-- The exotic feast's cutscene, as retail played it (Bob's Chronofoil capture b4ce5e3f.)
FEAST_CUTSCENE_FLAGS = 0xF8406EF3

NOTICE_BEAUTY = 0
NOTICE_FEAST = 1
NOTICE_HEROES = 2
HEROES_NEEDED = 5
-- What retail set UI8A to when a reward window was closed (Bob's capture b766138c, quest 840; 414 and 680.)
REWARD_CLOSED = 0x11

-- Sequence 3's five: the scenes of their talk, the line after (nil: not offered again), their nibble and BitFlag16.
HEROES = {
    { actor = ACTOR8, scenes = { 51, 52 }, after = nil, byte = 0, high = false, flag = 1 },
    { actor = ACTOR5, scenes = { 53 }, after = 54, byte = 1, high = true, flag = 2 },
    { actor = ACTOR4, scenes = { 55 }, after = 56, byte = 1, high = false, flag = 3 },
    { actor = ACTOR6, scenes = { 57 }, after = 58, byte = 2, high = true, flag = 4 },
    { actor = ACTOR7, scenes = { 59, 60 }, after = 61, byte = 2, high = false, flag = 5 },
}

-- Everyone's line in each sequence, as the client numbers them (its IsAcceptEvent order; objects have two scenes each,
-- the first played.)
LINES = {
    [1] = { [ACTOR2] = 4, [ACTOR3] = 5, [ACTOR4] = 6, [ACTOR5] = 7, [ACTOR6] = 8, [ACTOR7] = 9, [EOBJECT0] = 10,
            [EOBJECT1] = 12, [EOBJECT2] = 14, [EOBJECT3] = 16, [EOBJECT4] = 18, [EOBJECT5] = 20, [EOBJECT6] = 22,
            [EOBJECT7] = 24 },
    [2] = { [ACTOR2] = 28, [ACTOR1] = 29, [ACTOR3] = 30, [ACTOR4] = 31, [ACTOR5] = 32, [ACTOR6] = 33, [ACTOR7] = 34,
            [EOBJECT0] = 35, [EOBJECT1] = 37, [EOBJECT2] = 39, [EOBJECT3] = 41, [EOBJECT4] = 43, [EOBJECT5] = 45,
            [EOBJECT6] = 47, [EOBJECT7] = 49 },
    [3] = { [ACTOR2] = 62, [ACTOR1] = 63, [ACTOR3] = 64, [EOBJECT3] = 65, [EOBJECT4] = 67, [EOBJECT5] = 69,
            [EOBJECT6] = 71, [EOBJECT7] = 73 },
    [255] = { [ACTOR2] = 76, [ACTOR1] = 77, [ACTOR3] = 78, [ACTOR4] = 79, [ACTOR5] = 80, [ACTOR6] = 81, [ACTOR7] = 82,
              [EOBJECT3] = 83, [EOBJECT4] = 85, [EOBJECT5] = 87, [EOBJECT6] = 89, [EOBJECT7] = 91 },
}

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

-- BitFlag16: flag 1 is the top bit of UI8E, flag 9 the top bit of UI8F (as retail set them here.)
function hasFlag16(player, flag)
    local byte = 4 + (flag - 1) // 8
    return (player:get_quest_var(EVENT_ID, byte) & (0x80 >> ((flag - 1) % 8))) ~= 0
end

function setFlag16(player, flag)
    local byte = 4 + (flag - 1) // 8
    player:set_quest_var(EVENT_ID, byte, player:get_quest_var(EVENT_ID, byte) | (0x80 >> ((flag - 1) % 8)))
end

function heroOf(base_id)
    for _, hero in ipairs(HEROES) do
        if hero.actor == base_id then
            return hero
        end
    end
    return nil
end

function isObject(base_id)
    return base_id == EOBJECT0 or base_id == EOBJECT1 or base_id == EOBJECT2 or base_id == EOBJECT3
        or base_id == EOBJECT4 or base_id == EOBJECT5 or base_id == EOBJECT6 or base_id == EOBJECT7
        or base_id == EOBJECT8
end

-- The client's terms for the one step of sequences 1 and 2 (ACTOR1, then EOBJECT8): UI8AL < 1, BitFlag16 1 unset.
function stepOffered(player)
    return getVar(player, 0, false) < 1 and not hasFlag16(player, 1)
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- Scenes still to play in this event (the second of a hero's two), and the hero being talked to.
queued = {}
talking = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local hero = heroOf(BASE_ID)
    queued = {}
    talking = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 and stepOffered(player) then
        play(player, SCENE_BEAUTY, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT8 and stepOffered(player) then
        play(player, SCENE_FEAST, OBJECT_FLAGS)
    elseif sequence == 3 and hero ~= nil then
        if getVar(player, hero.byte, hero.high) == 0 and not hasFlag16(player, hero.flag) then
            talking = hero
            for i = 2, #hero.scenes do
                table.insert(queued, hero.scenes[i])
            end
            play(player, hero.scenes[1], HIDE_HOTBAR)
        elseif hero.after ~= nil then
            play(player, hero.after, HIDE_HOTBAR)
        else
            player:finish_event()
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], isObject(BASE_ID) and OBJECT_FLAGS or HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- One more of the five talked to.
function heroTalked(player, hero)
    local count = getVar(player, 0, true) + 1
    player:event_notice(EVENT_ID, NOTICE_HEROES, { count, HEROES_NEEDED })
    if count >= HEROES_NEEDED then
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:quest_sequence(EVENT_ID, 255)
        return
    end
    setVar(player, hero.byte, hero.high, 1)
    setVar(player, 0, true, count) -- UI8AH: the count
    setFlag16(player, hero.flag)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_OFFER_TALK, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_OFFER_TALK then
        play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_BEAUTY then
        player:event_notice(EVENT_ID, NOTICE_BEAUTY, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_FEAST then
        play(player, SCENE_FEAST_CUTSCENE, FEAST_CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_FEAST_CUTSCENE then
        player:event_notice(EVENT_ID, NOTICE_FEAST, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif talking ~= nil and #queued > 0 then
        play(player, table.remove(queued, 1), HIDE_HOTBAR)
        return
    elseif talking ~= nil then
        local hero = talking
        talking = nil
        heroTalked(player, hero)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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
