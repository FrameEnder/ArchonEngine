-- Quest: Kicking the Hornet's Nest
--
-- Written from the client's quest script (kawari-questdump, 2026-11-04) and checked against Bob's Chronofoil capture
-- 208c3716 (2026-11-03, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/256.json):
-- Sequence 0:   ACTOR0 (the guildmaster) offers it (scene 0, a yield: QuestOffer, 1 when accepted): sequence 1.
-- Sequence 1:   the four marauders ACTOR1-ACTOR4 (in Ul'dah): talked to, a line (scenes 1, 4, 7, 10); an emote at one
--               (the quest's emote listeners take any: retail's was Emote 23) plays its scene (2, 5, 8, 11, 0x2001: it
--               storms off) and counts it in UI8AL, with its bit in UI8F (0x80, 0x40, 0x20, 0x10: the client stops
--               offering it), notice 0 with [count, 4]. The fourth: the variables cleared, sequence 2. Talked to once
--               it's been seen off, its after line (3, 6, 9, 12). ACTOR5-ACTOR8, their idle companions, have empty
--               scenes 13-16 (not seen on retail.)
-- Sequence 2:   ACTOR9: talked to, a line (17); an emote at him plays CUT_SCENE_01 (scene 18, retail's flags
--               0x402400): notice 1, sequence 3.
-- Sequence 3:   ACTOR12 talks (scene 26): notice 2, sequence 4.
-- Sequence 4:   ACTOR0 talks (scene 27): notice 3, sequence 5.
-- Sequence 5:   ACTOR13 (in Western Thanalan) talks (scene 28): notice 4, sequence 6.
-- Sequence 6:   the caravan sacks EOBJECT1 and EOBJECT2: scene 32 or 34 (0x2001, empty), EVENT_ACTION_GATHER_SHORT's
--               bar (retail's 2 s), then scene 33 or 35 (0x2001, the "it appears" message): ENEMY0 or ENEMY1 comes out
--               (layer QB_ClsGla050_001 of the Western Thanalan drop-in, retail's positions, levels and HP). Retail
--               sets nothing for a search (it can be searched again while its enemy is about.) Each one defeated
--               gives an ITEM0: UI8AH counts them and UI8BL shows them (the client's GetEventItems), UI8AL = 1, its bit
--               in UI8F (0x80 for ENEMY0; 0x40 for ENEMY1 is taken to be, retail did ENEMY0 first), notice 5 with
--               [count, 2, ITEM0's icon]. The second: the variables cleared, UI8BH = 2 (ITEM0 in sequence 7),
--               sequence 7. ACTOR13 meanwhile: scene 36.
-- Sequence 7:   ACTOR13 takes the items (scene 40, NpcTrade in event mode: a yield, 1 when handed over), then scene
--               41: the variables cleared, notice 6, sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 45, a yield: QuestReward, 1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_SPOKEN_TO = 26
SCENE_GUILDMASTER = 27
SCENE_MERCHANT = 28
SCENE_MERCHANT_WAITS = 36
SCENE_TRADE = 40
SCENE_TRADED = 41
SCENE_REWARD = 45
SCENE_PFARMURL = 17
SCENE_PFARMURL_EMOTE = 18

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- CUT_SCENE_01's flags as retail played it (Bob's Chronofoil capture 208c3716.)
CUTSCENE_FLAGS = 0x402400

-- The four to see off in sequence 1: their scenes (talked to before, the emote, after) and their bit in UI8F.
MARAUDERS = {
    { actor = "ACTOR1", before = 1, emote = 2, after = 3, bit = 0x80 },
    { actor = "ACTOR2", before = 4, emote = 5, after = 6, bit = 0x40 },
    { actor = "ACTOR3", before = 7, emote = 8, after = 9, bit = 0x20 },
    { actor = "ACTOR4", before = 10, emote = 11, after = 12, bit = 0x10 },
}
-- Their companions' empty scenes in sequence 1 (by the client's order.)
COMPANIONS = { ACTOR5 = 13, ACTOR6 = 14, ACTOR7 = 15, ACTOR8 = 16 }
-- The sacks in sequence 6: their scenes (the search, then the message) and the enemy each brings out.
SACKS = {
    { eobject = "EOBJECT1", search = 32, appear = 33, enemy = "ENEMY0", bit = 0x80 },
    { eobject = "EOBJECT2", search = 34, appear = 35, enemy = "ENEMY1", bit = 0x40 },
}
FLAGS_BYTE = 5 -- UI8F

NOTICE_MARAUDER = 0
NOTICE_PFARMURL = 1
NOTICE_SPOKEN_TO = 2
NOTICE_GUILDMASTER = 3
NOTICE_MERCHANT = 4
NOTICE_ITEM = 5
NOTICE_TRADED = 6
MARAUDERS_NEEDED = 4
ITEMS_NEEDED = 2
-- ITEM0's icon, as retail's notice carried it (Bob's Chronofoil capture 208c3716.)
ITEM0_ICON = 25919

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function getNibble(player, byte, high)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        return (value >> 4) & 0x0F
    end
    return value & 0x0F
end

function setNibble(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

function clearVars(player)
    for byte = 0, 5 do
        player:set_quest_var(EVENT_ID, byte, 0)
    end
end

function hasBit(player, bit)
    return player:get_quest_var(EVENT_ID, FLAGS_BYTE) & bit ~= 0
end

function setBit(player, bit)
    player:set_quest_var(EVENT_ID, FLAGS_BYTE, player:get_quest_var(EVENT_ID, FLAGS_BYTE) | bit)
end

function marauder()
    for _, entry in ipairs(MARAUDERS) do
        if BASE_ID == _G[entry.actor] then
            return entry
        end
    end
    return nil
end

function sack()
    for _, entry in ipairs(SACKS) do
        if BASE_ID == _G[entry.eobject] then
            return entry
        end
    end
    return nil
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The marauder or sack this event is for.
current_entry = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and marauder() ~= nil then
        local entry = marauder()
        play(player, hasBit(player, entry.bit) and entry.after or entry.before, HIDE_HOTBAR)
    elseif sequence == 1 then
        for actor, scene in pairs(COMPANIONS) do
            if BASE_ID == _G[actor] then
                play(player, scene, HIDE_HOTBAR)
                return
            end
        end
        player:finish_event()
    elseif sequence == 2 and BASE_ID == ACTOR9 then
        play(player, SCENE_PFARMURL, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR12 then
        play(player, SCENE_SPOKEN_TO, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR0 then
        play(player, SCENE_GUILDMASTER, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR13 then
        play(player, SCENE_MERCHANT, HIDE_HOTBAR)
    elseif sequence == 6 and sack() ~= nil then
        current_entry = sack()
        play(player, current_entry.search, OBJECT_FLAGS)
    elseif sequence == 6 and BASE_ID == ACTOR13 then
        play(player, SCENE_MERCHANT_WAITS, HIDE_HOTBAR)
    elseif sequence == 7 and BASE_ID == ACTOR13 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local entry = marauder()
    if sequence == 1 and entry ~= nil and not hasBit(player, entry.bit) then
        current_entry = entry
        play(player, entry.emote, OBJECT_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR9 then
        play(player, SCENE_PFARMURL_EMOTE, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- A sack's bar finished: the scene after it.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 6 or current_entry == nil or current_entry.appear == nil then
        player:finish_event()
        return
    end
    play(player, current_entry.appear, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 6 then
        return
    end
    for _, entry in ipairs(SACKS) do
        if layout_id == _G[entry.enemy] and not hasBit(player, entry.bit) then
            local count = getNibble(player, 0, true) + 1
            player:event_notice(EVENT_ID, NOTICE_ITEM, { count, ITEMS_NEEDED, ITEM0_ICON })
            if count >= ITEMS_NEEDED then
                clearVars(player)
                setNibble(player, 1, true, ITEMS_NEEDED) -- UI8BH: ITEM0
                player:quest_sequence(EVENT_ID, 7)
            else
                setNibble(player, 0, true, count) -- UI8AH
                setNibble(player, 0, false, 1)    -- UI8AL
                setNibble(player, 1, false, count) -- UI8BL: ITEM0
                setBit(player, entry.bit)
            end
            return
        end
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local entry = current_entry
    current_entry = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif entry ~= nil and scene == entry.emote then
        local count = player:get_quest_var(EVENT_ID, 0) & 0x0F
        count = count + 1
        player:event_notice(EVENT_ID, NOTICE_MARAUDER, { count, MARAUDERS_NEEDED })
        if count >= MARAUDERS_NEEDED then
            clearVars(player)
            player:quest_sequence(EVENT_ID, 2)
        else
            setNibble(player, 0, false, count) -- UI8AL
            setBit(player, entry.bit)
        end
    elseif scene == SCENE_PFARMURL_EMOTE then
        player:event_notice(EVENT_ID, NOTICE_PFARMURL, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_SPOKEN_TO then
        player:event_notice(EVENT_ID, NOTICE_SPOKEN_TO, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_GUILDMASTER then
        player:event_notice(EVENT_ID, NOTICE_GUILDMASTER, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_MERCHANT then
        player:event_notice(EVENT_ID, NOTICE_MERCHANT, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif entry ~= nil and scene == entry.search then
        -- Searching the sack: the bar, then its message (see onEventActionCast.)
        current_entry = entry
        player:event_action(EVENT_ACTION_GATHER_SHORT)
        return
    elseif entry ~= nil and scene == entry.appear then
        player:spawn_quest_bnpc(_G[entry.enemy])
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        clearVars(player)
        player:event_notice(EVENT_ID, NOTICE_TRADED, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
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
