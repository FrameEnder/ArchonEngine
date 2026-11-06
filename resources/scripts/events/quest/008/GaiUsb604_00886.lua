-- Quest: The Unending War (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters: fixtures
-- tools/questkit/fixtures/886.json and `886-3c56e09f.json`; in 130088ec the second kill came while ACTOR1's scene 3
-- was open, so its fixture has it just before that talk):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   walking into EVENTRANGE0 (while UI8AL is under 2) plays scene 2 (its pop-up message, 0x2001), and as it
--               ends ENEMY0 and ENEMY1 (BNpcBase 724 and 725, layouts 4289901 and 4289905; drop-in layer
--               QB_GaiUsb604_001 of roc_r1f1, retail's positions, level 33, 1055 HP) come out for the player. Each one
--               down counts in UI8AL. ACTOR1 (offered while BitFlag8 1 is unset) plays scene 3 (0x2001; the client's
--               own warning if UI8AL is under 2): with both down, EVENT_ACTION_RESCUE_UNDER_MIDDLE's bar and scene 4
--               (0x2001, the NPC walking off): notice 0, sequence 2 with every variable 0; otherwise nothing.
--               EOBJECT0's scenes (5, 6) are empty: a search with nothing to show.
-- Sequence 2:   ACTOR0 (scene 7): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 (scene 8): notice 2, sequence 255.
-- Sequence 255: ACTOR3 rewards it (scene 9, a cutscene, 0xF8402EFB, a yield: QuestReward, 1; closing the window
--               leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_AMBUSH = 2
SCENE_ACTOR1 = 3
SCENE_RESCUED = 4
SCENE_SEARCH = 5
SCENE_SEARCHED = 6
SCENE_ACTOR0 = 7
SCENE_ACTOR2 = 8
SCENE_REWARD = 9

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
ENEMIES = { ENEMY0, ENEMY1 }
ENEMIES_TO_BEAT = 2
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The bar's next scene, while it runs.
after_bar = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function beaten(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F -- UI8AL
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, QUIET_FLAGS)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_SEARCH, QUIET_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 springs the ambush.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or beaten(player) >= ENEMIES_TO_BEAT then
        player:finish_event()
        return
    end
    play(player, SCENE_AMBUSH, QUIET_FLAGS)
end

function onEventActionCast(player)
    if after_bar == nil or current_scene ~= nil then
        player:finish_event()
        return
    end
    local scene = after_bar
    after_bar = nil
    play(player, scene, QUIET_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    for _, enemy in ipairs(ENEMIES) do
        if layout_id == enemy then
            player:set_quest_var(EVENT_ID, 0, beaten(player) + 1)
            return
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
    elseif scene == SCENE_AMBUSH then
        player:finish_event()
        -- Ones still out aren't brought out again.
        for _, enemy in ipairs(ENEMIES) do
            player:spawn_quest_bnpc(enemy)
        end
        return
    elseif scene == SCENE_ACTOR1 then
        if beaten(player) >= ENEMIES_TO_BEAT then
            after_bar = SCENE_RESCUED
            player:event_action(EVENT_ACTION_RESCUE_UNDER_MIDDLE)
            return
        end
    elseif scene == SCENE_SEARCH then
        after_bar = SCENE_SEARCHED
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_RESCUED then
        player:event_notice(EVENT_ID, 0, {})
        player:set_quest_var(EVENT_ID, 0, 0)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 255)
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
