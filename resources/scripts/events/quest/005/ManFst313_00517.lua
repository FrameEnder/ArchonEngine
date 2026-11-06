-- Quest: All Good Things (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 650dbfd8, 0fd8bce0 and 48aaf6fe (2026-11-06, the whole quest each: fixtures tools/questkit/fixtures/517.json,
-- `517-0fd8bce0.json`, `517-48aaf6fe.json`):
-- Sequence 0:   ACTOR0 (Y'shtola, by the Navel) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then talks
--               (scene 1, which ends with QuestAccepted; both 0x6001): sequence 1.
-- Sequence 1:   ACTOR1 (R'ashaht Rhiki, in Limsa Lominsa, scene 2: a cutscene, retail's 0xF8402FFF): notice 0,
--               sequence 2.
-- Sequence 2:   entering TERRITORYTYPE0 (212, the Waking Sands; onEnterTerritory) plays scene 3 (its fade in, 0x2000):
--               notice 1, sequence 3. The villagers there have lines meanwhile (ACTOR2 4, ACTOR3 5; ACTOR4's and
--               ACTOR5's scenes are empty.)
-- Sequence 3:   ACTOR6 (Noraxia, scene 8, a cutscene: 0xF8402EFB): notice 2, sequence 255.
-- Sequence 255: ACTOR7 (Iliud, in Eastern Thanalan) rewards it (scene 9, with its menu, a yield: QuestReward, 1), then
--               the cutscene (scene 10, 0xF8402EFB, parameter 0), and the quest completes as it ends. Retail then
--               completed With a Little Elbow Grease (871) too, with no rewards (FinishQuest 517, then 871, all three
--               runs): You Can't Take It with You (3867) stands in for the older quests it replaced.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_RASHAHT = 2
SCENE_WAKING_SANDS = 3
SCENE_NORAXIA = 8
SCENE_REWARD = 9
SCENE_CUTSCENE = 10

-- Retail's flags and parameter (Bob's Chronofoil captures of 2026-11-06.)
YSHTOLA_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001
RASHAHT_FLAGS = 0xF8402FFF
CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB
CUTSCENE_PARAMS = { 0 }

-- Completed with this one, as retail did.
QUEST_ALSO_COMPLETED = 871

LINES = {
    [2] = { [ACTOR2] = 4, [ACTOR3] = 5 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- The reward chosen in scene 9, kept until the cutscene after it ends.
chosen = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, YSHTOLA_FLAGS)
    elseif sequence == 1 and BASE_ID == ACTOR1 then
        play(player, SCENE_RASHAHT, RASHAHT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR6 then
        play(player, SCENE_NORAXIA, CUTSCENE_FLAGS)
    elseif sequence == 255 and BASE_ID == ACTOR7 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEnterTerritory(player)
    if player:get_quest_sequence(EVENT_ID) == 2 then
        play(player, SCENE_WAKING_SANDS, HIDE_HOTBAR)
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
            play(player, SCENE_ACCEPTED, YSHTOLA_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_RASHAHT then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_WAKING_SANDS then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_NORAXIA then
        player:event_notice(EVENT_ID, 2, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            chosen = results[2] or 0
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS, CUTSCENE_PARAMS)
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:finish_event()
        player:finish_quest(EVENT_ID, chosen)
        player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
