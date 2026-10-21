-- Quest: Relighting the Torch (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts; the client refuses it itself while QST_ACCEPT_CHECK is
--               accepted): sequence 1.
-- Sequence 1:   ACTOR1 (scene 1) gives ITEM0: notice 0 [21003], UI8BH = 1, sequence 2.
-- Sequence 2:   EOBJECT0: talking plays scene 2 (the client's prompt to use an item); using ITEM0 on it (a key item's
--               event, onEventItem) plays scene 3: notice 1, the variables cleared, sequence 3.
-- Sequence 3:   EOBJECT1: scene 5 (empty), EVENT_ACTION_ (32, the progress bar), scene 6 (a pop message), then ENEMY0
--               comes out (a layout ID, drop-in layer QB_SubSea106_001); killing it gives ITEM1: notice 2 [22623],
--               UI8BL = 1, sequence 4.
-- Sequence 4:   hand ITEM1 to ACTOR1 (scene 11, an NpcTrade: 1 if given), then scene 12: notice 3, sequence 5.
-- Sequence 5:   ACTOR0 (scene 17): notice 4, sequence 255.
-- Sequence 255: ACTOR2 rewards it (scene 18, a yield with the reward picked.)
-- Scenes 1, 11, 12, 17 and 18 with HIDE_HOTBAR, the objects' with HIDE_HOTBAR | NO_DEFAULT_CAMERA, as on retail.

SCENE_OFFER = 0
SCENE_ACTOR1 = 1
SCENE_USE_ITEM = 2
SCENE_ITEM_USED = 3
SCENE_EOBJECT1 = 5 -- empty in the client's script: the progress bar follows
SCENE_AMBUSH = 6
SCENE_TRADE = 11
SCENE_TRADED = 12
SCENE_ACTOR0 = 17
SCENE_FINISH = 18

OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
-- What retail's notices carried.
ITEM0_ICON = 21003
ITEM1_ICON = 22623

-- Retail's layout IDs for the objects, in case their base IDs aren't known.
LAYOUT_EOBJECT0 = 3929792
LAYOUT_EOBJECT1 = 3929797

function isObject(target, base, layout)
    return BASE_ID == base or (target ~= nil and target.object_id == layout)
end

function clearVars(player)
    for b = 0, 5 do
        player:set_quest_var(EVENT_ID, b, 0)
    end
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
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and isObject(target, EOBJECT0, LAYOUT_EOBJECT0) then
        play(player, SCENE_USE_ITEM, OBJECT_FLAGS)
    elseif sequence == 3 and isObject(target, EOBJECT1, LAYOUT_EOBJECT1) then
        play(player, SCENE_EOBJECT1, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR1 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 5 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if player:get_quest_sequence(EVENT_ID) == 2 and item_id == ITEM0 then
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) == 3 then
        play(player, SCENE_AMBUSH, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 3 or layout_id ~= ENEMY0 then
        return
    end
    player:event_notice(EVENT_ID, 2, { ITEM1_ICON })
    player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8BL = 1
    player:quest_sequence(EVENT_ID, 4)
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
        end
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, { ITEM0_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ITEM_USED then
        player:event_notice(EVENT_ID, 1, {})
        clearVars(player)
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_EOBJECT1 then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_)
        return
    elseif scene == SCENE_AMBUSH then
        -- Retail's order: the scene, its end, then ENEMY0 (brought out once at a time.)
        player:finish_event()
        player:spawn_quest_bnpc(ENEMY0)
        return
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:event_notice(EVENT_ID, 3, {})
        clearVars(player)
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_FINISH then
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
