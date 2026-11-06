-- Quest: The Heretic among Us (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec (2026-11-06, his second character, the quest battle won at the first try: fixture
-- tools/questkit/fixtures/941.json) and 3c56e09f (his first: two losses, then a win: `941-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   ACTOR1 (offered while UI8AL is 0 and BitFlag8 1 unset; scene 2, YesNoQuestBattle, a yield), or
--               walking into EVENTRANGE0 (while UI8BH is 0 and BitFlag8 6 unset; scene 9, 0x2001, the same question):
--               1 enters QUESTBATTLE0 (61) in TERRITORYTYPE0 (302), and it can be tried again after a loss (Bob's
--               first character turned it down twice by walking in, then lost twice before winning.) Winning it moves
--               to sequence 255 (content/questbattle/61.lua.) ACTOR2 and ACTOR3 (scenes 3, 4) and EOBJECT0 and
--               EOBJECT1 (5-8) have empty lines meanwhile.
-- Sequence 255: ACTOR4 rewards it (scene 10, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BATTLE = 2
SCENE_BATTLE_RANGE = 9
SCENE_REWARD = 10

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

LINES = {
    [1] = { [ACTOR2] = 3, [ACTOR3] = 4, [EOBJECT0] = 5, [EOBJECT1] = 7 },
}

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
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 asks too.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 1 then
        player:finish_event()
        return
    end
    play(player, SCENE_BATTLE_RANGE, QUIET_FLAGS)
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
    elseif scene == SCENE_BATTLE or scene == SCENE_BATTLE_RANGE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
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
