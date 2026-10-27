-- Quest: Sylphie's Trials (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b23b91ef (2026-10-26,
-- the whole quest; its fixture is tools/questkit/fixtures/93.json).
-- Three times over, in the Central Shroud: search the corrupted earth, air and water (EOBJECT0-2: their scene, then
-- EVENT_ACTION_SEARCH's bar, then the sheet's ..._EVENTACTIONOK scene, a message; all 0x2001, as retail), which
-- brings out a tainted sprite (ENEMY0-2, layout IDs: the Central Shroud drop-in's QB_ClsCnj004_001 layer, from
-- retail's spawns; owned by the player, naming the object as its trigger). Killing it sends a notice and moves on,
-- and Sylphie (ACTOR1-3, one for each spot) has a word before (SEQ_n_ACTORm) and sends you on after.
-- Sequence 0:   ACTOR0 (E-Sumi-Yan) offers the quest (scene 0, a yield returning 1 when accepted): sequence 1.
-- Sequence 1:   EOBJECT0 brings out ENEMY0; its kill: notice 0, sequence 2. ACTOR1 meanwhile: scene 2.
-- Sequence 2:   ACTOR1 (scene 3): notice 1, sequence 3.
-- Sequence 3:   EOBJECT1 brings out ENEMY1; its kill: notice 2, sequence 4. ACTOR2 meanwhile: scene 5.
-- Sequence 4:   ACTOR2 (scene 6): notice 3, sequence 5.
-- Sequence 5:   EOBJECT2 brings out ENEMY2; its kill: notice 4, sequence 6. ACTOR3 meanwhile: scene 8.
-- Sequence 6:   ACTOR3 (scene 9): notice 5, sequence 255.
-- Sequence 255: ACTOR0's reward (scene 10, retail's flags 0xF8402EFB; a yield returning 1 and the item chosen.)

SCENE_OFFER = 0
SCENE_SEARCH_EARTH = 1
SCENE_SEARCH_AIR = 4
SCENE_SEARCH_WATER = 7
SCENE_REWARD = 10

-- Retail's flags (Bob's Chronofoil capture b23b91ef.)
SEARCH_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
CUTSCENE_FLAGS = 0xF8402EFB

-- The three spots, by the sequence that searches them: the object, its scenes, what it brings out, Sylphie's
-- scenes there (meanwhile and after the kill, in the next sequence), and the notices for the kill and for her.
SPOTS = {
    [1] = { object = EOBJECT0, search = SCENE_SEARCH_EARTH, found = SEQ_1_EOBJECT0_EVENTACTIONOK or 100, enemy = ENEMY0,
            sylphie = ACTOR1, waiting = SEQ_1_ACTOR1 or 2, after = SEQ_2_ACTOR1 or 3, killed = 0, told = 1 },
    [3] = { object = EOBJECT1, search = SCENE_SEARCH_AIR, found = SEQ_3_EOBJECT1_EVENTACTIONOK or 98, enemy = ENEMY1,
            sylphie = ACTOR2, waiting = SEQ_3_ACTOR2 or 5, after = SEQ_4_ACTOR2 or 6, killed = 2, told = 3 },
    [5] = { object = EOBJECT2, search = SCENE_SEARCH_WATER, found = SEQ_5_EOBJECT2_EVENTACTIONOK or 96, enemy = ENEMY2,
            sylphie = ACTOR3, waiting = SEQ_5_ACTOR3 or 8, after = SEQ_6_ACTOR3 or 9, killed = 4, told = 5 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local spot = sequence ~= nil and SPOTS[sequence] or nil
    local told = sequence ~= nil and SPOTS[sequence - 1] or nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif spot ~= nil and BASE_ID == spot.object then
        -- The object's scene, the progress bar (see sceneFinished), then the "found it" scene once that finishes.
        play(player, spot.search, SEARCH_FLAGS)
    elseif spot ~= nil and BASE_ID == spot.sylphie then
        play(player, spot.waiting, SEARCH_FLAGS)
    elseif told ~= nil and BASE_ID == told.sylphie then
        play(player, told.after, SEARCH_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    local spot = SPOTS[player:get_quest_sequence(EVENT_ID) or -1]
    if spot == nil then
        player:finish_event()
        return
    end
    play(player, spot.found, SEARCH_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local spot = sequence ~= nil and SPOTS[sequence] or nil
    if spot == nil or layout_id ~= spot.enemy then
        return
    end
    player:event_notice(EVENT_ID, spot.killed, {})
    player:quest_sequence(EVENT_ID, sequence + 1)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local sequence = player:get_quest_sequence(EVENT_ID)
    local spot = sequence ~= nil and SPOTS[sequence] or nil
    local told = sequence ~= nil and SPOTS[sequence - 1] or nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif spot ~= nil and scene == spot.search then
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif spot ~= nil and scene == spot.found then
        player:spawn_quest_bnpc(spot.enemy)
    elseif told ~= nil and scene == told.after then
        player:event_notice(EVENT_ID, told.told, {})
        player:quest_sequence(EVENT_ID, sequence == 6 and 255 or sequence + 1)
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
