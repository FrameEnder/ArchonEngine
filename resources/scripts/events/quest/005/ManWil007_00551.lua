-- Quest: Duty, Honor, Country (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 551):
-- Sequence 0:   ACTOR0 offers the quest (scene 0, a yield returning 1 when accepted), then a cutscene (scene 1) and
--               scene 2 (QuestAccepted.) Accepting gives you ITEM0 (UI8BH = 1.)
-- Sequence 1:   ACTOR1, outside TERRITORYTYPE0, asks whether to go in (scene 3, a yes/no yield returning 1): yes
--               takes you to POPRANGE0 in TERRITORYTYPE0 (retail: PrepareZoning, then EventFinish.) There ACTOR2
--               takes ITEM0 (hand-over scene 4, a yield returning 1 when handed over), then a cutscene (scene 5):
--               notice 0 [1, 1], UI8BH = 0, sequence 2. (Leaving TERRITORYTYPE0 is its exit's own warp event.)
-- Sequence 2:   EOBJECT0 in Central Thanalan: its scene 7 (which only waits), the Waiting bar (event action 13,
--               EVENT_ACTION_WAITING3_LONG, about 10 s), then scene 8 (0x2001, YesNoQuestBattle, a yield returning
--               1): QUESTBATTLE0 in TERRITORYTYPE1. Winning it moves the quest to sequence 3; losing leaves it here
--               to try again (Bob lost twice before winning.) ACTOR2 has a line (scene 9.)
-- Sequence 3:   coming back into TERRITORYTYPE2 plays the aftermath cutscene (scene 10, an enter-territory event):
--               sequence 255.
-- Sequence 255: ACTOR1 takes you into TERRITORYTYPE0 again (scene 11, as scene 3); ACTOR3 rewards it (scene 12, a
--               yield with the chosen item), then the closing cutscenes (scene 13, given the chosen item, which it
--               returns.) ACTOR2 has a line (scene 14.) Every variable stays 0 from sequence 2 on, apart from UI8AH and
--               UI8AL, which retail set to 1 as scene 13 began.

SCENE_OFFER = 0
SCENE_OFFER_CUTSCENE = 1
SCENE_ACCEPTED = 2
SCENE_GO_IN = 3
SCENE_TRADE = 4
SCENE_TRADE_OK = 5
SCENE_DESTINATION = 7 -- only waits; the Waiting bar comes after it
SCENE_ENTER_BATTLE = 8
SCENE_OWYNE = 9
SCENE_AFTERMATH = 10
SCENE_GO_IN_AGAIN = 11
SCENE_REWARD = 12
SCENE_FINISH = 13
SCENE_OWYNE_FINISH = 14

NOTICE_DELIVERED = 0

-- Scene flags as retail sent them (Bob's Chronofoil capture of 2026-10-18): 0x2000 for talking, 0xF8402EFB for the
-- cutscenes, 0x2001 for the quest battle's yes/no.
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA
EVENT_ACTION_WAIT = EVENT_ACTION_WAITING3_LONG or 13

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
chosen_item = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_GO_IN, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == EOBJECT0 then
        play(player, SCENE_DESTINATION, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_OWYNE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_GO_IN_AGAIN, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_OWYNE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The Waiting bar at EOBJECT0 ended: the quest battle's yes/no.
function onEventActionCast(player)
    play(player, SCENE_ENTER_BATTLE, OBJECT_FLAGS)
end

-- Back in Central Thanalan after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 3 then
        play(player, SCENE_AFTERMATH, CUTSCENE_FLAGS)
    else
        player:finish_event()
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_OFFER_CUTSCENE, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_OFFER_CUTSCENE then
        play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
        return
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
    elseif scene == SCENE_GO_IN or scene == SCENE_GO_IN_AGAIN then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
    elseif scene == SCENE_TRADE then
        -- 1 when ITEM0 was handed over.
        if results[1] == 1 then
            play(player, SCENE_TRADE_OK, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_TRADE_OK then
        player:event_notice(EVENT_ID, NOTICE_DELIVERED, { 1, 1 })
        player:set_quest_var(EVENT_ID, 1, 0) -- UI8BH = 0: ITEM0 is gone
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_DESTINATION then
        player:event_action(EVENT_ACTION_WAIT) -- continues in onEventActionCast
        return
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE1)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            chosen_item = results[2] or 0
            play(player, SCENE_FINISH, CUTSCENE_FLAGS, { chosen_item })
            -- Retail set UI8AH and UI8AL to 1 as the closing cutscenes began (Bob's Chronofoil capture.)
            player:set_quest_var(EVENT_ID, 0, 0x11)
            return
        end
    elseif scene == SCENE_FINISH then
        player:finish_quest(EVENT_ID, chosen_item)
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
