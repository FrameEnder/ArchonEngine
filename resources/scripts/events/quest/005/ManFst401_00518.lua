-- Quest: Eyes on Me (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/518.json and `518-3c56e09f.json`):
-- Sequence 0:   ACTOR0 (in Eastern Thanalan) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then talks
--               (scene 1, which ends with QuestAccepted): sequence 1.
-- Sequence 1:   EOBJECT0 (offered while BitFlag8 1 is unset): its empty scene 2 (0x2001), EVENT_ACTION_SEARCH_MIDDLE's
--               bar, then scene 3 (its pop-up message, 0x2001); as it ends ENEMY0 (BNpcBase 1902, layout 4296987)
--               comes out for the player (0.5-0.7 s after on retail; drop-in layer QB_ManFst401_001 of wil_w1f3, its
--               position, level 35 and 1830 HP.) Retail set nothing for the search. ENEMY0 down: notice 0 [ITEM0's
--               icon], sequence 2 with UI8BH = 1 (ITEM0.)
-- Sequence 2:   ACTOR0 (scene 4): notice 1, sequence 255.
-- Sequence 255: ACTOR1 takes ITEM0 (scene 5, NpcTrade, a yield: 1 when handed over), rewards it (scene 6, a yield:
--               QuestReward, 1 and the optional item picked; closing the window leaves UI8A 0x11), then the cutscene
--               (scene 7, 0xF8402EFB, its parameter the item picked), and the quest completes as it ends.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_SEARCH = 2
SCENE_SEARCHED = 3
SCENE_ACTOR0 = 4
SCENE_TRADE = 5
SCENE_REWARD = 6
SCENE_CUTSCENE = 7

SEARCH_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
-- ITEM0's icon, on the notice as ENEMY0 falls (Bob's Chronofoil captures of 2026-11-06.)
ITEM0_ICON = 30407
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The reward picked in scene 6, kept until the cutscene after it ends.
chosen = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_SEARCH, SEARCH_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar at EOBJECT0 finished.
function onEventActionCast(player)
    if current_scene ~= nil or player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_SEARCHED, SEARCH_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) == 1 and layout_id == ENEMY0 then
        player:event_notice(EVENT_ID, 0, { ITEM0_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
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
    elseif scene == SCENE_SEARCH then
        -- The bar, then scene 3 (see onEventActionCast.)
        player:event_action(EVENT_ACTION_SEARCH_MIDDLE)
        return
    elseif scene == SCENE_SEARCHED then
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY0)
        return
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            chosen = results[2] or 0
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS, { chosen })
            return
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
        end
    elseif scene == SCENE_CUTSCENE then
        player:finish_event()
        player:finish_quest(EVENT_ID, chosen)
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
