-- Quest: The Lominsan Way
--
-- Written from the client's quest script (kawari-questdump, 2026-11-03) and checked against Bob's Chronofoil captures
-- 44df02fc (2026-11-03, the whole quest, the battle lost five times first) and c9145fdc (2026-11-03, the whole quest):
-- its fixtures are tools/questkit/fixtures/811.json and 811-c9145fdc.json.
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene where the client
--               accepts it (scene 90, SEQ_0_ACTOR0_LQ, 0xF8402EFB): sequence 1.
-- Sequence 1:   ACTOR1 asks whether to enter QUESTBATTLE0 (scene 2: YesNoQuestBattle, a yield, 1 for yes): into
--               TERRITORYTYPE0. Winning it moves the quest to sequence 255 (content/questbattle/59.lua); losing leaves
--               it here, to ask again.
-- Sequence 255: ACTOR2 (in Eastern La Noscea) rewards it (scene 5, a cutscene and a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 90
SCENE_ENTER_BATTLE = 2
SCENE_REWARD = 5

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

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
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
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
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
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
