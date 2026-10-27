-- Quest: Like Mother, Like Daughter (Gridania)
--
-- Checked against the client's quest script (kawari-questdump) and Bob's Chronofoil capture b23b91ef (2026-10-26,
-- the whole quest and two runs of its quest battle, one lost; its fixture is tools/questkit/fixtures/440.json).
-- Sequence 0:   ACTOR0 (E-Sumi-Yan) offers the quest (scene 0, a yield returning 1 when accepted), then scene 1
--               (the client accepts it there): sequence 1.
-- Sequence 1:   ACTOR1 (Wulfiue, in the South Shroud, scene 2): notice 0, sequence 2. ACTOR2 (Nolanel) and ACTOR3
--               (Joacin) have a line each (scenes 3 and 4).
-- Sequence 2:   ACTOR4 (Sylphie, scene 5, a cutscene; retail's flags 0xF8402EF3): notice 1, sequence 3. ACTOR2,
--               ACTOR3 and ACTOR1 have lines (6, 7, 8).
-- Sequence 3:   ACTOR5 (Sylphie, scene 11: YesNoQuestBattle; 1 enters QUESTBATTLE0 in TERRITORYTYPE0, and she can be
--               asked again after a loss). Winning it moves to sequence 4 (content/questbattle/56.lua.)
-- Sequence 4:   ACTOR12 (Sylphie, scene 24): notice 3, sequence 255. ACTOR13-15 have lines (28, 29, 30).
-- Sequence 255: ACTOR0's reward (scene 31, retail's flags 0xF8402EFB; a yield returning 1 and the item chosen.)
--               ACTOR12-15 have lines (35-38).
-- The client's other scenes (9, 10, 12-23, 25-27, 32-34: the onlookers and the objects) do nothing.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_WULFIUE = 2
SCENE_SYLPHIE = 5
SCENE_ENTER_BATTLE = 11
SCENE_AFTER_BATTLE = 24
SCENE_REWARD = 31

-- Retail's flags (Bob's Chronofoil capture b23b91ef.)
SYLPHIE_CUTSCENE_FLAGS = 0xF8402EF3
CUTSCENE_FLAGS = 0xF8402EFB

NOTICE_WULFIUE = 0
NOTICE_SYLPHIE = 1
NOTICE_AFTER_BATTLE = 3

-- The lines the others have in each sequence: { [sequence] = { [base ID] = scene } }.
SCENE_NOLANEL_1 = 3
SCENE_JOACIN_1 = 4
SCENE_NOLANEL_2 = 6
SCENE_JOACIN_2 = 7
SCENE_WULFIUE_2 = 8
SCENE_NOLANEL_4 = 28
SCENE_JOACIN_4 = 29
SCENE_WULFIUE_4 = 30
SCENE_SYLPHIE_DONE = 35
SCENE_NOLANEL_DONE = 36
SCENE_JOACIN_DONE = 37
SCENE_WULFIUE_DONE = 38
LINES = {
    [1] = { [ACTOR2] = SCENE_NOLANEL_1, [ACTOR3] = SCENE_JOACIN_1 },
    [2] = { [ACTOR2] = SCENE_NOLANEL_2, [ACTOR3] = SCENE_JOACIN_2, [ACTOR1] = SCENE_WULFIUE_2 },
    [4] = { [ACTOR13] = SCENE_NOLANEL_4, [ACTOR14] = SCENE_JOACIN_4, [ACTOR15] = SCENE_WULFIUE_4 },
    [255] = { [ACTOR12] = SCENE_SYLPHIE_DONE, [ACTOR13] = SCENE_NOLANEL_DONE, [ACTOR14] = SCENE_JOACIN_DONE,
              [ACTOR15] = SCENE_WULFIUE_DONE },
}

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
        play(player, SCENE_WULFIUE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR4 then
        play(player, SCENE_SYLPHIE, SYLPHIE_CUTSCENE_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR5 then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR12 then
        play(player, SCENE_AFTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
    elseif line ~= nil then
        play(player, line, HIDE_HOTBAR)
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
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_WULFIUE then
        player:event_notice(EVENT_ID, NOTICE_WULFIUE, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_SYLPHIE then
        player:event_notice(EVENT_ID, NOTICE_SYLPHIE, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTER_BATTLE then
        player:event_notice(EVENT_ID, NOTICE_AFTER_BATTLE, {})
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
