-- Quest: Terror at Fallgourd (today's)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- 412ba91d (2026-11-01, the whole quest: its fixture is tools/questkit/fixtures/3863.json):
-- Sequence 0:   ACTOR0 (Minfilia) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Terror at Fallgourd (784), is accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   ACTOR1 (Noraxia) talks (scene 2): notice 0, sequence 2.
-- Sequence 2:   ACTOR2 (Medrod) talks (scene 3): notice 1, sequence 255.
-- Sequence 255: ACTOR2 again: scene 4 (the client's hint) when talked to; the right emote on him (retail's 35, /soothe)
--               plays the reward cutscene (scene 5, 0xF8402EFB, a yield: QuestReward, 1 and the optional item
--               chosen), any other the client's scene 6. Retail then completed the old Terror at Fallgourd (784) too,
--               with no rewards: FinishQuest 784 with 3863's.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_NORAXIA = 2
SCENE_MEDROD = 3
SCENE_HINT = 4
SCENE_EMOTE_RIGHT = 5
SCENE_EMOTE_WRONG = 6

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
-- The emote retail used on Medrod (Emote 35, /soothe; Bob's Chronofoil capture 412ba91d.)
EMOTE_SOOTHE = 35

NOTICE_NORAXIA = 0
NOTICE_MEDROD = 1
-- The quest this one replaced, completed with it (retail's FinishQuest 784 with 3863's.)
QUEST_ALSO_COMPLETED = 784

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
        play(player, SCENE_NORAXIA, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_MEDROD, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_HINT, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    if player:get_quest_sequence(EVENT_ID) == 255 and BASE_ID == ACTOR2 then
        if emote == EMOTE_SOOTHE then
            play(player, SCENE_EMOTE_RIGHT, CUTSCENE_FLAGS)
        else
            play(player, SCENE_EMOTE_WRONG, HIDE_HOTBAR)
        end
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
    elseif scene == SCENE_NORAXIA then
        player:event_notice(EVENT_ID, NOTICE_NORAXIA, {})
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_MEDROD then
        player:event_notice(EVENT_ID, NOTICE_MEDROD, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_EMOTE_RIGHT then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
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
