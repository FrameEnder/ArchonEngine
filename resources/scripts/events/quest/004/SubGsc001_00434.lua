-- Quest: It Could Happen to You
--
-- Written from the client's quest script (kawari-questdump, 2026-10-29) and checked against Bob's Chronofoil capture
-- deaf7e28 (2026-10-29, the whole quest: its fixture is tools/questkit/fixtures/434.json). It opens the Gold Saucer:
-- the Ul'dahn ticketer's flight there (Warp 108, WarpCondition 29) asks for this quest done.
-- Sequence 0:   ACTOR0 (in Ul'dah) offers it (scene 0, a yield returning 1 when accepted), then scene 1
--               (QuestAccepted): sequence 255 with ITEM0, the event item, in UI8BH (retail's vars 0x10.)
-- Sequence 255: ACTOR2 (Elyenora, the airship ticketer) takes ITEM0 (scene 3: an event-mode NpcTrade, a yield
--               returning 1 once handed over), asks to go (scene 4, a yes/no: no cancels the scene), then the
--               cutscene (scene 5, 0xF8402EFB: the flight, the reward, a yield returning 1) takes you to the Gold
--               Saucer (POPRANGE0 in territory 144; retail: the zone change, then FinishQuest, then EventFinish.)
-- ACTOR0's line at sequence 255 is scene 6. ACTOR1's scenes (2, 7) are empty.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_TRADE = 3
SCENE_YES_NO = 4
SCENE_FLIGHT = 5
SCENE_ACTOR0_LINE = 6
SCENE_ACTOR1_FINISH = 7

-- The Gold Saucer, where POPRANGE0 is (retail's zone change in Bob's Chronofoil capture deaf7e28.)
GOLD_SAUCER = 144

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB, retail's
-- A yes/no answered No (its error code.)
ERROR_DECLINED = 50

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags or HIDE_HOTBAR, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER)
    elseif sequence == nil and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0_LINE)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1_FINISH)
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
            play(player, SCENE_ACCEPTED)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 255)
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
    elseif scene == SCENE_TRADE then
        -- 1 once ITEM0 was handed over (an event item: it lives in UI8BH.)
        if results[1] == 1 then
            play(player, SCENE_YES_NO)
            return
        end
    elseif scene == SCENE_YES_NO then
        -- Saying no cancels the scene (CancelEventScene: error code 50); retail answered [0, 0] for yes.
        if player.scene_error_code ~= ERROR_DECLINED then
            play(player, SCENE_FLIGHT, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_FLIGHT then
        if results[1] == 1 then
            player:change_territory_pop_range(GOLD_SAUCER, POPRANGE0)
            player:finish_quest(EVENT_ID, 0)
            player:finish_event()
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
