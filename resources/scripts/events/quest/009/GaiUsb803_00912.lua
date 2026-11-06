-- Quest: In the Eyes of Gods and Men (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-06) and checked against Bob's Chronofoil captures
-- 130088ec and 3c56e09f (2026-11-06, the whole quest on each of his two characters, its quest battle won at the first
-- try: fixtures tools/questkit/fixtures/912.json and `912-3c56e09f.json`):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1 (QuestAccepted):
--               sequence 1.
-- Sequence 1:   ACTOR1 (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR0 (scene 3): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 (offered while UI8AL is 0 and BitFlag16 1 unset; scene 4, YesNoQuestBattle, a yield), or
--               walking into EVENTRANGE0 (while UI8BH is 0 and BitFlag16 13 unset; scene 18, 0x2001, the same
--               question): 1 enters QUESTBATTLE0 (60) in TERRITORYTYPE0 (301); declining leaves it to be asked again
--               (Bob's first character turned both down first.) Winning it moves to sequence 255
--               (content/questbattle/60.lua.) EOBJECT0 and EOBJECT1 (scenes 5-8) and ACTOR3-11 (9-17) have empty
--               lines meanwhile.
-- Sequence 255: ACTOR0 rewards it (scene 19, a yield: QuestReward, 1; closing the window leaves UI8A 0x11.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ACTOR1 = 2
SCENE_ACTOR0 = 3
SCENE_BATTLE = 4
SCENE_BATTLE_RANGE = 18
SCENE_REWARD = 19

QUIET_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855.)
REWARD_CLOSED = 0x11

LINES = {
    [3] = { [EOBJECT0] = 5, [EOBJECT1] = 7, [ACTOR3] = 9, [ACTOR4] = 10, [ACTOR5] = 11, [ACTOR6] = 12, [ACTOR7] = 13,
            [ACTOR8] = 14, [ACTOR9] = 15, [ACTOR10] = 16, [ACTOR11] = 17 },
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
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- Walking into EVENTRANGE0 asks too.
function onEnterTrigger(player, arg)
    if player:get_quest_sequence(EVENT_ID) ~= 3 then
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
    elseif scene == SCENE_ACTOR1 then
        player:event_notice(EVENT_ID, 0, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_ACTOR0 then
        player:event_notice(EVENT_ID, 1, {})
        player:quest_sequence(EVENT_ID, 3)
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
