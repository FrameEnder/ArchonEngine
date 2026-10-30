-- Quest: Presence of the Enemy
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/719.json):
-- Sequence 0:   ACTOR0 (Komuxio) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   four to talk to, in any order (the client offers each while UI8AL < 4 and its flag is unset):
--                 ACTOR1 (Pelixia) scene 2, BitFlag8 1; ACTOR2 (Nathaxio) scene 3, flag 2;
--                 ACTOR3 (Monne) scene 4, flag 3; ACTOR4 (Victor) scene 5, flag 4
--               UI8AL counts them, notice 0 [count, 4]; after the fourth retail cleared the variables and moved to
--               sequence 2.
-- Sequence 2:   ACTOR0 talks (scene 6): notice 1, sequence 3.
-- Sequence 3:   walking into EVENTRANGE0 (the client allows it while UI8AL < 4) plays scene 7 (0x2001, a pop-up
--               message) and ENEMY0-3 come out (layout IDs; one of each at a time.) Each kill counts in UI8AL, with no
--               notice (retail: 1, 2, 3, 4.) EOBJECT0, the suspicious box: scene 8 (0x2001; only a message while
--               UI8AL < 4.) With all four down retail then ran EVENT_ACTION_GATHER_MIDDLE's bar on it (ActorControlTarget
--               EventAction 1, 7, five seconds), then scene 9 (0x2001, empty): notice 2 [1, 1, ITEM0's icon], UI8AL
--               cleared, ITEM0 given (UI8BH = 1; the client lists ITEM0 from UI8BH), sequence 255.
-- Sequence 255: ACTOR0 takes ITEM0 (scene 12, NpcTrade in event mode: a yield, 1 when handed over) and rewards it
--               (scene 13, a yield: QuestReward, 1 and the chosen item.)
-- The four enemies are layer QB_GaiUsa105_001 of the East Shroud drop-in, at retail's positions, levels and HP.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_KOMUXIO = 6
SCENE_AMBUSH = 7
SCENE_BOX = 8
SCENE_BOX_SEARCHED = 9
SCENE_TRADE = 12
SCENE_REWARD = 13
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_TALKED = 0
NOTICE_KOMUXIO = 1
NOTICE_BOX = 2
TALKS_NEEDED = 4
KILLS_NEEDED = 4
-- ITEM0's icon, as retail's notice carried it (Bob's Chronofoil capture 5fcc970c.)
ITEM0_ICON = 26151

TALKS = {
    { actor = ACTOR1, scene = 2, flag = 1 },
    { actor = ACTOR2, scene = 3, flag = 2 },
    { actor = ACTOR3, scene = 4, flag = 3 },
    { actor = ACTOR4, scene = 5, flag = 4 },
}

ENEMIES = { ENEMY0, ENEMY1, ENEMY2, ENEMY3 }

-- UI8AL: the people talked to in sequence 1, the kills in sequence 3.
function count(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F
end

function setCount(player, value)
    player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | value)
end

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

function isEnemy(layout_id)
    for _, enemy in ipairs(ENEMIES) do
        if layout_id == enemy then
            return true
        end
    end
    return false
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_talk = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_talk = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 then
        for _, talk in ipairs(TALKS) do
            if BASE_ID == talk.actor and not hasFlag(player, talk.flag) then
                current_talk = talk
                play(player, talk.scene, HIDE_HOTBAR)
                return
            end
        end
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_KOMUXIO, HIDE_HOTBAR)
        return
    elseif sequence == 3 and BASE_ID == EOBJECT0 then
        play(player, SCENE_BOX, OBJECT_FLAGS)
        return
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

-- Walking into EVENTRANGE0 brings the four out (one of each at a time: nothing new while they're out.)
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 3 or count(player) >= KILLS_NEEDED then
        player:finish_event()
        return
    end
    for _, enemy in ipairs(ENEMIES) do
        player:spawn_quest_bnpc(enemy)
    end
    play(player, SCENE_AMBUSH, OBJECT_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if not isEnemy(layout_id) or player:get_quest_sequence(EVENT_ID) ~= 3 or count(player) >= KILLS_NEEDED then
        return
    end
    setCount(player, count(player) + 1)
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 3 then
        player:finish_event()
        return
    end
    play(player, SCENE_BOX_SEARCHED, OBJECT_FLAGS)
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
    elseif current_talk ~= nil and scene == current_talk.scene then
        local talked = count(player) + 1
        player:event_notice(EVENT_ID, NOTICE_TALKED, { talked, TALKS_NEEDED })
        if talked >= TALKS_NEEDED then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 2)
        else
            setCount(player, talked)
            setFlag(player, current_talk.flag)
        end
        current_talk = nil
    elseif scene == SCENE_KOMUXIO then
        player:event_notice(EVENT_ID, NOTICE_KOMUXIO, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_BOX then
        if count(player) >= KILLS_NEEDED then
            -- Searching the box: the bar, then scene 9 (see onEventActionCast.)
            player:event_action(EVENT_ACTION_GATHER_MIDDLE)
            return
        end
    elseif scene == SCENE_BOX_SEARCHED then
        player:event_notice(EVENT_ID, NOTICE_BOX, { 1, 1, ITEM0_ICON })
        for byte = 0, 5 do
            player:set_quest_var(EVENT_ID, byte, 0)
        end
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
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
