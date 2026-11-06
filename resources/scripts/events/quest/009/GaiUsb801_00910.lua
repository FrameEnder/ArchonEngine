-- Quest: Road to Redemption (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/910.json and `910-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   ACTOR1 (offered while BitFlag8 1 is unset; scene 2): as it ends ENEMY0 (BNpcBase 1904, layout 4293113;
--               drop-in layer QB_GaiUsb801_001 of roc_r1f1, retail's position, level 38, 2272 HP) comes out for the
--               player (0.2-0.4 s after on retail). It down: notice 0, sequence 2. ACTOR2-4 have lines meanwhile
--               (scenes 3-5.)
-- Sequence 2:   ACTOR2, 3 and 4, in any order, each once (BitFlag8 1-3) while UI8AL (the count) is under 3: its empty
--               scene, EVENT_ACTION_RESCUE_UNDER_SHORT's bar, its second scene (the NPC running off; all 0x2001;
--               ACTOR2 6 and 7, ACTOR3 8 and 9, ACTOR4 10 and 11): UI8AL up one, notice 1 [count, 3]; the third:
--               sequence 3 with every variable 0. ACTOR1 has a line meanwhile (scene 12.)
-- Sequence 3:   ACTOR1: scene 13, EVENT_ACTION_RESCUE_UNDER_MIDDLE's bar, scene 14 (both 0x2001): notice 2, sequence
--               255.
-- Sequence 255: ACTOR0 rewards it (scene 15, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR1_RESCUE = 13
SCENE_REWARD = 15

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

-- Sequence 2's three: their first scene (the second follows it) and BitFlag8.
RESCUES = {
    [ACTOR2] = { scene = 6, flag = 1 },
    [ACTOR3] = { scene = 8, flag = 2 },
    [ACTOR4] = { scene = 10, flag = 3 },
}
NEEDED = 3
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

LINES = {
    [1] = { [ACTOR2] = 3, [ACTOR3] = 4, [ACTOR4] = 5 },
    [2] = { [ACTOR1] = 12 },
}

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The first scene of the one being helped, while its bar runs.
helping = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local rescue = RESCUES[BASE_ID]

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 and not hasFlag(player, 1) then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and rescue ~= nil and not hasFlag(player, rescue.flag) then
        helping = rescue
        play(player, rescue.scene, QUIET_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        helping = { scene = SCENE_ACTOR1_RESCUE }
        play(player, SCENE_ACTOR1_RESCUE, QUIET_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if helping == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    play(player, helping.scene + 1, QUIET_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) == 1 and layout_id == ENEMY0 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
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
    elseif scene == SCENE_ACTOR1 then
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY0)
        return
    elseif helping ~= nil and scene == helping.scene then
        if scene == SCENE_ACTOR1_RESCUE then
            player:event_action(EVENT_ACTION_RESCUE_UNDER_MIDDLE)
        else
            player:event_action(EVENT_ACTION_RESCUE_UNDER_SHORT)
        end
        return
    elseif helping ~= nil and scene == helping.scene + 1 then
        local rescue = helping
        helping = nil
        if scene == SCENE_ACTOR1_RESCUE + 1 then
            player:event_notice(EVENT_ID, 2, {})
            player:quest_sequence(EVENT_ID, 255)
        else
            local count = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1 -- UI8AL
            player:event_notice(EVENT_ID, 1, { count, NEEDED })
            if count >= NEEDED then
                player:set_quest_var(EVENT_ID, 0, 0)
                player:set_quest_var(EVENT_ID, 5, 0)
                player:quest_sequence(EVENT_ID, 3)
            else
                player:set_quest_var(EVENT_ID, 0, count)
                setFlag(player, rescue.flag)
            end
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
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
