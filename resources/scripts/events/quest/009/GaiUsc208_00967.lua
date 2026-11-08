-- Quest: History Repeating (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08 (d7993cf0, 195176f8 and 0d10bf45, each the whole quest; fixtures
-- tools/questkit/fixtures/967.json, 967-195176f8.json, 967-0d10bf45.json):
-- Sequence 0:   ACTOR0 (Davyd) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Mimidoa) talks, walks off and fades (scene 2, 0x6001): ITEM0 (UI8BH = 1), notice 0 with its
--               icon, sequence 2.
-- Sequence 2:   ACTOR2 (Mimidoa, further on) talks (scene 3): notice 1, sequence 3. (The client script still lists
--               the two campfires, EOBJECT0 and EOBJECT1, for this sequence, with empty scenes 4-7; its
--               IsTargetingPossible makes them untargetable here, and retail went straight on.)
-- Sequence 3:   EOBJECT1 (the campfire, layout 4300218): scene 8 (0x2001, empty), EVENT_ACTION_WAITING2_MIDDLE's bar
--               (5 s on retail), then scene 9 (0x2001, YesNoQuestBattle): 1 enters QUESTBATTLE0 (86) in
--               TERRITORYTYPE0 (330). Winning it moves the quest to sequence 4 with ITEM0 gone
--               (content/questbattle/86.lua.) ACTOR2 has a line meanwhile (14).
-- Sequence 4:   ACTOR2 talks, walks off and fades (scene 15, 0x6001): notice 3, sequence 5.
-- Sequence 5:   ACTOR0 (Davyd) talks (scene 16): ITEM1 (UI8BH = 1), notice 4 with its icon, sequence 255.
-- Sequence 255: ACTOR3 (Ceana) takes ITEM1 (scene 17, NpcTrade in event mode: a yield, 1 when handed over) and
--               rewards it (scene 18, a yield: QuestReward, 1.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_FIRST_TALK = 2
SCENE_SECOND_TALK = 3
SCENE_BATTLE_WAIT = 8
SCENE_BATTLE = 9
SCENE_BEFORE_BATTLE_LINE = 14
SCENE_AFTER_BATTLE = 15
SCENE_DAVYD = 16
SCENE_TRADE = 17
SCENE_REWARD = 18

WALK_OFF_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA | INVINCIBLE -- 0x6001 on retail (Mimidoa walks off and fades)
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_ITEM0 = 0
NOTICE_SECOND_TALK = 1
NOTICE_AFTER_BATTLE = 3
NOTICE_ITEM1 = 4
-- The items' icons, as retail's notices carried them (Bob's Chronofoil captures of 2026-11-08.)
ITEM0_ICON = 25919
ITEM1_ICON = 22603

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11

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
        play(player, SCENE_FIRST_TALK, WALK_OFF_FLAGS)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_SECOND_TALK, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == EOBJECT1 then
        play(player, SCENE_BATTLE_WAIT, OBJECT_FLAGS)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_BEFORE_BATTLE_LINE, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR2 then
        play(player, SCENE_AFTER_BATTLE, WALK_OFF_FLAGS)
    elseif sequence == 5 and BASE_ID == ACTOR0 then
        play(player, SCENE_DAVYD, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR3 then
        -- The trade first, every time (ITEM1 stays held until the quest completes.)
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

-- The bar at EOBJECT1 finished: the question.
function onEventActionCast(player)
    if player:get_quest_sequence(EVENT_ID) ~= 3 then
        player:finish_event()
        return
    end
    play(player, SCENE_BATTLE, OBJECT_FLAGS)
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
    elseif scene == SCENE_FIRST_TALK then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        player:event_notice(EVENT_ID, NOTICE_ITEM0, { ITEM0_ICON })
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_SECOND_TALK then
        player:event_notice(EVENT_ID, NOTICE_SECOND_TALK, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_BATTLE_WAIT then
        -- Waiting there: the bar, then the question (see onEventActionCast.)
        player:event_action(EVENT_ACTION_WAITING2_MIDDLE)
        return
    elseif scene == SCENE_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
    elseif scene == SCENE_AFTER_BATTLE then
        player:event_notice(EVENT_ID, NOTICE_AFTER_BATTLE, {})
        player:quest_sequence(EVENT_ID, 5)
    elseif scene == SCENE_DAVYD then
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM1
        player:event_notice(EVENT_ID, NOTICE_ITEM1, { ITEM1_ICON })
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
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
