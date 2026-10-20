-- Quest 641 (Ul'dah's story, between Duty, Honor, Country and A Royal Reception)
--
-- Written from retail only (Bob's Chronofoil capture of 2026-10-18, questkit replay 641): no kawari-questdump of it
-- yet, so the NPCs are compared by the layout IDs retail talked to (the script name is a guess; the server finds any
-- `*_00641.lua` in this folder.) Dump it in the Quest Lab to check the scene numbers and branches.
-- Sequence 0:   Owyne (layout 4094165, in territory 210) offers it (scene 0, a yield returning 1 when accepted), then
--               scene 1, after which it's accepted: sequence 1.
-- Sequence 1:   Momodi (layout 3146810) (scene 3): notice 0, sequence 2.
-- Sequence 2:   the NPC at layout 3985683 (the Steps of Thal) gives an item (scene 5): notice 1 with its icon (21002 on
--               retail), UI8BH = 1, sequence 255.
-- Sequence 255: Momodi (scene 6, a yield returning 1), then the reward (scene 7, a yield: 1; gil only on retail.)

OWYNE = 4094165
MOMODI = 3146810
GIVER = 3985683

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_MOMODI = 3
SCENE_GIVER = 5
SCENE_REPORT = 6
SCENE_FINISH = 7

NOTICE_MOMODI = 0
NOTICE_ITEM = 1
ITEM_ICON = 21002 -- what retail's notice carried

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    local npc = target and target.object_id

    if sequence == nil and npc == OWYNE then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 1 and npc == MOMODI then
        play(player, SCENE_MOMODI, HIDE_HOTBAR)
    elseif sequence == 2 and npc == GIVER then
        play(player, SCENE_GIVER, HIDE_HOTBAR)
    elseif sequence == 255 and npc == MOMODI then
        play(player, SCENE_REPORT, HIDE_HOTBAR)
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
    elseif scene == SCENE_MOMODI then
        player:event_notice(EVENT_ID, NOTICE_MOMODI)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_GIVER then
        player:event_notice(EVENT_ID, NOTICE_ITEM, { ITEM_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: the item
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_REPORT then
        if results[1] == 1 then
            play(player, SCENE_FINISH, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_FINISH then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
