-- Quest: What's in the Box (the Arcanists' level 5 quest)
--
-- Written from the client's quest script (kawari-questdump, Bob's dump of 2026-11-12) and checked against Bob's Kawari
-- Recorder export of 2026-11-12 (the whole quest; its fixture is tools/questkit/fixtures/455.json: sequences,
-- variables and notices, no scenes). The recording has no scene numbers: they're the listing's, in the order the
-- quest's listeners come (each object a scene before its bar and one after, as Bob's captured quests had them). The
-- Recorder caught scenes 8 and 9 in a field fight: 0x2001 and 0xF8486EFB, which the script uses.
-- Sequence 0:   ACTOR0 (Thubyrgeim) offers it (scene 0, QuestOffer: 1 accepts), then scene 1 (QuestAccepted).
-- Sequence 1:   three each of ENEMY0 (tempered orators, UI8AL, notice 0) and ENEMY1 (midge swarms, UI8BH, notice 1),
--               notices with [count, 3]; a kind at 3 counts in UI8AH; both done: sequence 2, every variable cleared.
-- Sequence 2:   ACTOR0 (scene 2): notice 2, sequence 3.
-- Sequence 3:   EOBJECT0 (the practice crates): scene 3, EVENT_ACTION_SEARCH's bar, scene 4 (a ScenarioMessage), and
--               ENEMY2 and ENEMY3 (tree slugs, layer QB_ClsAcn050_001 of sea_s1f1, retail's positions, level and HP
--               from the recording) come out. Each down counts in UI8AL (the client's IsAcceptEvent counts them below
--               2); the second: notice 3, sequence 4, UI8AL cleared. EOBJECT1 (the other crates) has scenes 5 and 6
--               and nothing else.
-- Sequence 4:   EOBJECT2 (a practice crate): scene 7, EVENT_ACTION_PROCESS_UPPER_MIDDLE's bar, scene 8, then ITEM0 in
--               UI8BH (where the client's GetEventItems lists it), notice 4 [21003], sequence 255, and K'lyhia's
--               cutscene (scene 9). EOBJECT1 has scenes 10 and 11.
-- Sequence 255: ACTOR0 takes ITEM0 (scene 12, NpcTrade: 1 when handed over), then the reward (scene 13, QuestReward.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TALK = 2
SCENE_CRATES = 3
SCENE_CRATES_FOUND = 4
SCENE_OTHER_CRATES = 5
SCENE_OTHER_CRATES_AFTER = 6
SCENE_CRATE = 7
SCENE_CRATE_FOUND = 8
SCENE_KLYHIA = 9
SCENE_OTHER_CRATES_LATER = 10
SCENE_OTHER_CRATES_LATER_AFTER = 11
SCENE_TRADE = 12
SCENE_REWARD = 13

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
CUTSCENE_FLAGS = 0xF8486EFB -- retail's (the Recorder export's field fight at 09:53 caught scenes 8 and 9)

-- Sequence 1's enemies: their todo (and notice), and where each count is kept (byte, high nibble.)
ENEMIES = {
    [ENEMY0] = { notice = 0, byte = 0, high = false }, -- UI8AL
    [ENEMY1] = { notice = 1, byte = 1, high = true }, -- UI8BH
}
KILLS_NEEDED = 3
KINDS = 2
SLUGS = { ENEMY2, ENEMY3 }
ICON_ITEM0 = 21003 -- notice 4's argument (Bob's recording)

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

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The object whose bar is running: its scene after the bar.
searching = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_TALK, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == EOBJECT0 then
        play(player, SCENE_CRATES, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        play(player, SCENE_OTHER_CRATES, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == EOBJECT2 then
        play(player, SCENE_CRATE, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == EOBJECT1 then
        play(player, SCENE_OTHER_CRATES_LATER, OBJECT_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- A bar finished: the object's scene after it.
function onEventActionCast(player)
    if searching == nil then
        player:finish_event()
        return
    end
    local scene = searching
    searching = nil
    play(player, scene, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 1 then
        local enemy = ENEMIES[base_id]
        if enemy == nil then
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
        local done = getVar(player, 0, true) + 1 -- UI8AH
        if done >= KINDS then
            player:set_quest_var(EVENT_ID, 0, 0)
            player:set_quest_var(EVENT_ID, 1, 0)
            player:quest_sequence(EVENT_ID, 2)
            return
        end
        setVar(player, enemy.byte, enemy.high, count)
        setVar(player, 0, true, done)
    elseif sequence == 3 and (layout_id == ENEMY2 or layout_id == ENEMY3) then
        local count = getVar(player, 0, false) + 1
        if count >= #SLUGS then
            player:event_notice(EVENT_ID, 3, {})
            setVar(player, 0, false, 0)
            player:quest_sequence(EVENT_ID, 4)
        else
            setVar(player, 0, false, count)
        end
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_TALK then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_CRATES then
        searching = SCENE_CRATES_FOUND
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_CRATES_FOUND then
        player:finish_event()
        -- Ones still out aren't brought out again.
        for _, slug in ipairs(SLUGS) do
            player:spawn_quest_bnpc(slug)
        end
        return
    elseif scene == SCENE_OTHER_CRATES then
        play(player, SCENE_OTHER_CRATES_AFTER, OBJECT_FLAGS)
        return
    elseif scene == SCENE_OTHER_CRATES_LATER then
        play(player, SCENE_OTHER_CRATES_LATER_AFTER, OBJECT_FLAGS)
        return
    elseif scene == SCENE_CRATE then
        searching = SCENE_CRATE_FOUND
        player:event_action(EVENT_ACTION_PROCESS_UPPER_MIDDLE)
        return
    elseif scene == SCENE_CRATE_FOUND then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- ITEM0 in UI8BH
        player:event_notice(EVENT_ID, 4, { ICON_ITEM0 })
        player:quest_sequence(EVENT_ID, 255)
        play(player, SCENE_KLYHIA, CUTSCENE_FLAGS)
        return
    elseif scene == SCENE_TRADE then
        -- 1 when the item was handed over, nothing when the window was closed.
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
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
