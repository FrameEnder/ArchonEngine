-- Quest: Trial by Water (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b23b91ef (2026-10-26,
-- the whole quest and two runs of its quest battle, one lost; its fixture is tools/questkit/fixtures/92.json).
-- Sequence 0:   ACTOR0 (E-Sumi-Yan) offers the quest (scene 0, a yield returning 1 when accepted; the client
--               accepts it in the same scene): sequence 1, with ITEM0 as a key item (UI8BH, the client's
--               GetEventItems.)
-- Sequence 1:   EOBJECT0, the corrupted water (scene 1: the client opens the key item window); using ITEM0 on it
--               (event type 8, scene 2's message) brings out ENEMY0, a tainted water sprite (a layout ID: the East
--               Shroud drop-in's QB_ClsCnj003_001 layer). Killing it takes the key item: notice 0, sequence 2.
-- Sequence 2:   ACTOR1 (Joacin, scene 7, a cutscene): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 (Aniud, scene 8): notice 2, sequence 4.
-- Sequence 4:   ACTOR3, the wounded Wood Wailer, asks for help (scene 9); ACTION0 (Cure) on him (event type 18, the
--               ActionResult event, scene 10): notice 3, sequence 5. ACTOR5, the harried healer, has a line meanwhile
--               (scene 13).
-- Sequence 5:   ACTOR5 (scene 14): notice 4, sequence 6. ACTOR3 thanks you again (scene 15.)
-- Sequence 6:   ACTOR6 (the Gods' Quiver bow) offers the quest battle (scene 17, YesNoQuestBattle; 1 enters
--               QUESTBATTLE0 in TERRITORYTYPE0; retail let him be asked again after a loss), as does walking into
--               EVENTRANGE0 (scene 34). Winning it moves to sequence 7 (content/questbattle/16.lua.) ACTOR7
--               (Sylphie, scene 18), ACTOR3 (scene 20) and ACTOR5 (scene 21) have lines meanwhile.
-- Sequence 7:   ACTOR18 (Sylphie again, scene 35): notice 6, sequence 255.
-- Sequence 255: ACTOR0's reward (scene 36, retail's flags 0xF8402EFB; a yield returning 1 and the item chosen.)
-- The client's other scenes (3, 4, 6, 11, 12, 16, 19, 22-33) do nothing, and 5 is scene 2's message again.

SCENE_OFFER = 0
SCENE_CORRUPTED_WATER = 1
SCENE_ITEM_USED = 2
SCENE_JOACIN = 7
SCENE_ANIUD = 8
SCENE_WOUNDED = 9
SCENE_HEALED = 10
SCENE_HEALER_WAITING = 13
SCENE_HEALER = 14
SCENE_WOUNDED_AFTER = 15
SCENE_ENTER_BATTLE = 17
SCENE_SYLPHIE = 18
SCENE_WOUNDED_BEFORE_BATTLE = 20
SCENE_HEALER_BEFORE_BATTLE = 21
SCENE_RANGE_BATTLE = 34
SCENE_SYLPHIE_AFTER_BATTLE = 35
SCENE_REWARD = 36

-- Retail's flags (Bob's Chronofoil capture b23b91ef.)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
CUTSCENE_FLAGS = 0xF8402EFB

NOTICE_DEFEATED = 0
NOTICE_JOACIN = 1
NOTICE_ANIUD = 2
NOTICE_HEALED = 3
NOTICE_HEALER = 4
NOTICE_SYLPHIE = 6

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- UI8BH: ITEM0 held.
function setKeyItem(player, count)
    local value = player:get_quest_var(EVENT_ID, 1)
    player:set_quest_var(EVENT_ID, 1, (value & 0x0F) | (count << 4))
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == EOBJECT0 then
        play(player, SCENE_CORRUPTED_WATER, OBJECT_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR1 then
        play(player, SCENE_JOACIN, CUTSCENE_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_ANIUD, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR3 then
        play(player, SCENE_WOUNDED, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR5 then
        play(player, SCENE_HEALER_WAITING, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR5 then
        play(player, SCENE_HEALER, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR3 then
        play(player, SCENE_WOUNDED_AFTER, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == ACTOR6 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == ACTOR7 then
        play(player, SCENE_SYLPHIE, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == ACTOR3 then
        play(player, SCENE_WOUNDED_BEFORE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 6 and BASE_ID == ACTOR5 then
        play(player, SCENE_HEALER_BEFORE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 7 and BASE_ID == ACTOR18 then
        play(player, SCENE_SYLPHIE_AFTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

-- ITEM0 used on the corrupted water.
function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == EOBJECT0 and player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS)
        return
    end
    player:finish_event()
end

-- Cure on the wounded Wood Wailer.
function onActionResult(action_id, target, player)
    if action_id == ACTION0 and BASE_ID == ACTOR3 and player:get_quest_sequence(EVENT_ID) == 4 then
        play(player, SCENE_HEALED, OBJECT_FLAGS)
        return
    end
    player:finish_event()
end

-- Walking up to the bowmen offers the battle too.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) == 6 then
        play(player, SCENE_RANGE_BATTLE, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onBNpcKill(base_id, layout_id, player)
    if layout_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 1 then
        return
    end
    setKeyItem(player, 0)
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
    player:quest_sequence(EVENT_ID, 2)
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            setKeyItem(player, 1)
        end
    elseif scene == SCENE_ITEM_USED then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_JOACIN then
        player:event_notice(EVENT_ID, NOTICE_JOACIN, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ANIUD then
        player:event_notice(EVENT_ID, NOTICE_ANIUD, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_HEALED then
        player:event_notice(EVENT_ID, NOTICE_HEALED, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_HEALER then
        player:event_notice(EVENT_ID, NOTICE_HEALER, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif scene == SCENE_ENTER_BATTLE or scene == SCENE_RANGE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_SYLPHIE_AFTER_BATTLE then
        player:event_notice(EVENT_ID, NOTICE_SYLPHIE, {})
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
