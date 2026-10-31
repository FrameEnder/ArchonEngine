-- Quest: Helping Horn
--
-- Written from the client's quest script (kawari-questdump, 2026-10-31) and checked against Bob's Chronofoil capture
-- a0149e5e (2026-10-31, the whole quest: its fixture is tools/questkit/fixtures/774.json):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the client
--               accepts it: sequence 1.
-- Sequence 1:   ENEMY0 is a BNpcBase (any of the open world's): each one defeated gives ITEM0, counted in UI8BH (the
--               client lists ITEM0 from it), notice 0 [count, 4, ITEM0's icon]; retail's four kills all counted. At
--               four, sequence 2 (UI8BH stays 4.)
-- Sequence 2:   ACTOR0 takes ITEM0 (scene 2, NpcTrade in event mode: a yield, 1 when handed over), then scene 3:
--               notice 1, sequence 3.
-- Sequence 3:   ACTOR1 takes ITEM0 the same way (scene 4) and gives ITEM1 (scene 5): notice 2 [ITEM1's icon],
--               sequence 255, UI8BL = 1 (ITEM1 held) and nothing else.
-- Sequence 255: ACTOR2 takes ITEM1 (scene 6, NpcTrade) and rewards it (scene 7, a yield: QuestReward, 1 and the
--               optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE_ACTOR0 = 2
SCENE_AFTER_ACTOR0 = 3
SCENE_TRADE_ACTOR1 = 4
SCENE_AFTER_ACTOR1 = 5
SCENE_TRADE_ACTOR2 = 6
SCENE_REWARD = 7

NOTICE_KILL = 0
NOTICE_ACTOR0 = 1
NOTICE_ACTOR1 = 2
KILLS_NEEDED = 4
-- ITEM0's and ITEM1's icons, as retail's notices carried them (Bob's Chronofoil capture a0149e5e.)
ITEM0_ICON = 22204
ITEM1_ICON = 22603

-- UI8BH: the kills (and ITEM0 held.)
function kills(player)
    return player:get_quest_var(EVENT_ID, 1) >> 4
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE_ACTOR2, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or base_id ~= ENEMY0 then
        return
    end
    local count = kills(player) + 1
    player:event_notice(EVENT_ID, NOTICE_KILL, { count, KILLS_NEEDED, ITEM0_ICON })
    player:set_quest_var(EVENT_ID, 1, count << 4)
    if count >= KILLS_NEEDED then
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
    elseif scene == SCENE_TRADE_ACTOR0 then
        if results[1] == 1 then
            play(player, SCENE_AFTER_ACTOR0, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_AFTER_ACTOR0 then
        player:event_notice(EVENT_ID, NOTICE_ACTOR0, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_TRADE_ACTOR1 then
        if results[1] == 1 then
            play(player, SCENE_AFTER_ACTOR1, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_AFTER_ACTOR1 then
        player:event_notice(EVENT_ID, NOTICE_ACTOR1, { ITEM1_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8BL: ITEM1 held, ITEM0 gone
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE_ACTOR2 then
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
