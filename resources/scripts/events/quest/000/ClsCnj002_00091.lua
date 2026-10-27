-- Quest: Trial by Wind (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b23b91ef (2026-10-26,
-- the whole quest and one run of its quest battle; its fixture is tools/questkit/fixtures/91.json).
-- Sequence 0:   ACTOR0 (E-Sumi-Yan) offers the quest (scene 0, a yield returning 1 when accepted), then scene 1 shows
--               it accepted: sequence 1.
-- Sequence 1:   ACTOR1 (Wulfiue, in the East Shroud) explains (scene 2): notice 0, sequence 2.
-- Sequence 2:   search EOBJECT0 (scene 3, then EVENT_ACTION_SEARCH's bar, then scene 4's message; all 0x2001). It brings
--               out ENEMY0, a honeybee swarm (a layout ID: the East Shroud drop-in's QB_ClsCnj002_001 layer), as
--               retail sent it just after the event finished. Killing it sends notice 1, sequence 3.
-- Sequence 3:   ACTOR1 again (scene 6): notice 2, sequence 4.
-- Sequence 4:   search EOBJECT1, the corrupted air (scene 7, the bar, then scene 8: YesNoQuestBattle, 1 enters
--               QUESTBATTLE0 in TERRITORYTYPE0). Winning the battle moves to sequence 5 (content/questbattle/54.lua).
-- Sequence 5:   ACTOR1 (scene 9): notice 4, sequence 255.
-- Sequence 255: ACTOR0's reward (scene 10, retail's flags 0xF8402EFB; a yield returning 1 and the item chosen.) The
--               Quest sheet's reward unlocks the Armoury System (unlock 6), given by finish_quest.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_WULFIUE = 2
SCENE_SEARCH = 3
SCENE_FOUND = 4
SCENE_WULFIUE_AGAIN = 6
SCENE_CORRUPTED_AIR = 7
SCENE_ENTER_BATTLE = 8
SCENE_AFTER_BATTLE = 9
SCENE_REWARD = 10

-- Retail's flags (Bob's Chronofoil capture b23b91ef.)
SEARCH_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
CUTSCENE_FLAGS = 0xF8402EFB

NOTICE_EXPLAINED = 0
NOTICE_DEFEATED = 1
NOTICE_TOLD = 2
NOTICE_BATTLE_REPORTED = 4

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The search scene that plays once the progress bar finishes.
after_search = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_WULFIUE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        after_search = SCENE_FOUND
        play(player, SCENE_SEARCH, SEARCH_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR1 then
        play(player, SCENE_WULFIUE_AGAIN, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == EOBJECT1 then
        after_search = SCENE_ENTER_BATTLE
        play(player, SCENE_CORRUPTED_AIR, SEARCH_FLAGS)
    elseif sequence == 5 and BASE_ID == ACTOR1 then
        play(player, SCENE_AFTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    if after_search == nil then
        player:finish_event()
        return
    end
    local scene = after_search
    after_search = nil
    play(player, scene, SEARCH_FLAGS)
end

function onBNpcKill(base_id, layout_id, player)
    if layout_id ~= ENEMY0 or player:get_quest_sequence(EVENT_ID) ~= 2 then
        return
    end
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, {})
    player:quest_sequence(EVENT_ID, 3)
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
    elseif scene == SCENE_WULFIUE then
        player:event_notice(EVENT_ID, NOTICE_EXPLAINED, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_SEARCH or scene == SCENE_CORRUPTED_AIR then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_SEARCH)
        return
    elseif scene == SCENE_FOUND then
        player:spawn_quest_bnpc(ENEMY0)
    elseif scene == SCENE_WULFIUE_AGAIN then
        player:event_notice(EVENT_ID, NOTICE_TOLD, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTER_BATTLE then
        player:event_notice(EVENT_ID, NOTICE_BATTLE_REPORTED, {})
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
