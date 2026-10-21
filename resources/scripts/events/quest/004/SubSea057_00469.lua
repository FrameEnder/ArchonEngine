-- Quest: Just Deserts (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21; the quest battle, content/questbattle/29.lua, from the same capture):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: the client's QuestOffer, 1 when accepted), then the cutscene
--               (scene 1), where the client accepts it; accepted after it: sequence 1.
-- Sequence 1:   ACTOR1 (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (scene 3, he runs off): notice 1, sequence 3.
-- Sequence 3:   ACTOR4: scene 10 (empty), EVENT_ACTION_ (35, the progress bar), scene 11: notice 2, sequence 4.
-- Sequence 4:   ACTOR5: scene 18 (empty), EVENT_ACTION_, then the quest battle's yes/no (scene 19, a yield:
--               YesNoQuestBattle, 1 enters QUESTBATTLE0 in TERRITORYTYPE0). Winning it moves the quest to sequence 5.
-- Sequence 5:   back in TERRITORYTYPE1, the aftermath cutscenes (scene 26, an enter-territory event): notice 4,
--               sequence 6.
-- Sequence 6:   ACTOR0 (scene 27, a cutscene): notice 5, sequence 7.
-- Sequence 7:   ACTOR0 (scene 29): notice 6, sequence 255.
-- Sequence 255: ACTOR10 rewards it (scene 30, a yield: QuestReward, 1 completes it; the server grants the unlocks.)
-- The other NPCs about have a line each, from the client's script (not played on retail): ACTOR3, ACTOR4, ACTOR5 and
-- ACTOR6..ACTOR8 in sequence 2 (scenes 4, 5, 6 and 7), ACTOR3, ACTOR6..ACTOR8 and ACTOR5 in sequence 3 (13, 14 and 17),
-- ACTOR6..ACTOR8 and ACTOR3 in sequence 4 (20 and 23), ACTOR9 in sequence 6 (28); the client's other scenes for them
-- are empty, and so are EOBJECT0's.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR2 = 3
SCENE_ACTOR4 = 10 -- empty in the client's script: the progress bar follows
SCENE_ACTOR4_DONE = 11
SCENE_ACTOR5 = 18 -- empty: the progress bar follows
SCENE_ENTER_BATTLE = 19
SCENE_AFTERMATH = 26
SCENE_ACTOR0_REPORT = 27
SCENE_ACTOR0_NEXT = 29
SCENE_FINISH = 30

-- The one-line talks, by sequence and NPC (the client's script.)
LINES = {
    [2] = { [ACTOR3] = 4, [ACTOR4] = 5, [ACTOR5] = 6, [ACTOR6] = 7, [ACTOR7] = 7, [ACTOR8] = 7 },
    [3] = { [ACTOR3] = 13, [ACTOR6] = 14, [ACTOR7] = 14, [ACTOR8] = 14, [ACTOR5] = 17 },
    [4] = { [ACTOR6] = 20, [ACTOR7] = 20, [ACTOR8] = 20, [ACTOR3] = 23 },
    [6] = { [ACTOR9] = 28 },
}

-- Scene flags as retail sent them.
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
REWARD_FLAGS = SET_BASE | HIDE_HOTBAR | DISABLE_SKIP -- 0xF8482EFB

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local line = sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] or nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_ACTOR2, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR4 then
        play(player, SCENE_ACTOR4, OBJECT_FLAGS)
    elseif sequence == 4 and BASE_ID == ACTOR5 then
        play(player, SCENE_ACTOR5, OBJECT_FLAGS)
    elseif sequence == 6 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0_REPORT, CUTSCENE_FLAGS)
    elseif sequence == 7 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0_NEXT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR10 then
        play(player, SCENE_FINISH, REWARD_FLAGS)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventActionCast(player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    if sequence == 3 then
        play(player, SCENE_ACTOR4_DONE, OBJECT_FLAGS)
    elseif sequence == 4 then
        play(player, SCENE_ENTER_BATTLE, OBJECT_FLAGS)
    else
        player:finish_event()
    end
end

-- Back in Middle La Noscea after winning the battle.
function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 5 then
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR2 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ACTOR4 or scene == SCENE_ACTOR5 then
        -- The event carries on once the progress bar finishes, see onEventActionCast.
        player:event_action(EVENT_ACTION_)
        return
    elseif scene == SCENE_ACTOR4_DONE then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTERMATH then
        player:event_notice(EVENT_ID, 4, {})
        player:quest_sequence(EVENT_ID, 6)
    elseif scene == SCENE_ACTOR0_REPORT then
        player:event_notice(EVENT_ID, 5, {})
        player:quest_sequence(EVENT_ID, 7)
    elseif scene == SCENE_ACTOR0_NEXT then
        player:event_notice(EVENT_ID, 6, {})
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
