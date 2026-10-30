-- Quest: Druthers House Rules
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/3859.json):
-- Sequence 0:   ACTOR0 (Buscarron) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Druthers House Rules (726), is accepted), then scene 1, where the
--               client accepts it, and ITEM0 is given (UI8BH = 1: the client lists ITEM0 from UI8BH): sequence 1.
-- Sequence 1:   ACTOR1 (the mead-soaked Midlander): talking plays scene 2 (0x2001), which only opens the inventory;
--               ITEM0 used on him (the client allows it while BitFlag8 1 is unset) plays scene 3 (0x2001, he walks off
--               and fades): notice 0, ITEM0 gone, sequence 255. ACTOR2 (the drunken Duskwight) has a line (scene 4.)
-- Sequence 255: ACTOR0 rewards it (scene 5, a yield: QuestReward, 1); ACTOR2 has another line (scene 6.) Retail then
--               completed the old Druthers House Rules (726) too: FinishQuest 3859, then 726.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_MIDLANDER = 2
SCENE_ITEM_USED = 3
SCENE_DUSKWIGHT = 4
SCENE_REWARD = 5
SCENE_DUSKWIGHT_AFTER = 6
OBJECT_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail

NOTICE_ITEM_USED = 0
-- The quest this one replaced, completed with it (retail's FinishQuest 726 right after 3859's.)
QUEST_ALSO_COMPLETED = 726

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
        play(player, SCENE_MIDLANDER, OBJECT_FLAGS)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_DUSKWIGHT, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_DUSKWIGHT_AFTER, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEventItem(item_id, target, player)
    if item_id == ITEM0 and BASE_ID == ACTOR1 and player:get_quest_sequence(EVENT_ID) == 1 then
        play(player, SCENE_ITEM_USED, OBJECT_FLAGS)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
    elseif scene == SCENE_ITEM_USED then
        player:event_notice(EVENT_ID, NOTICE_ITEM_USED, {})
        player:set_quest_var(EVENT_ID, 1, 0) -- ITEM0 handed over
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
