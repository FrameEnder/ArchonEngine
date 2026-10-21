-- Quest: Plowshares to Swords (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts): sequence 1.
-- Sequence 1:   three kills of ENEMY0 or ENEMY1 (the Quest sheet's listeners are drops; on retail every kill gave one:
--               three of ENEMY1): each gives ITEM0, notice 0 with [count, 3, 21002]; UI8AL counts them, UI8BH is
--               ITEM0 held. The third leaves only UI8BH (3), with sequence 2.
-- Sequence 2:   hand the items to ACTOR0 (scene 1, an NpcTrade: 1 if given), then scene 2: notice 1, sequence 255.
-- Sequence 255: ACTOR1: the reward (scene 4, a yield: 1 completes it.)

SCENE_OFFER = 0
SCENE_TRADE = 1
SCENE_TRADED = 2
SCENE_FINISH = 4

NOTICE_DROP = 0
NOTICE_TRADED = 1
NEEDED = 3
-- The drop notice's third argument, as retail sent it (probably ITEM0's icon.)
ITEM_ICON = 21002

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
    elseif sequence == 2 and BASE_ID == ACTOR0 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or (base_id ~= ENEMY0 and base_id ~= ENEMY1) then
        return
    end
    local count = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1
    player:event_notice(EVENT_ID, NOTICE_DROP, { count, NEEDED, ITEM_ICON })
    if count >= NEEDED then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:set_quest_var(EVENT_ID, 1, count << 4)
        player:quest_sequence(EVENT_ID, 2)
    else
        player:set_quest_var(EVENT_ID, 0, count)
        player:set_quest_var(EVENT_ID, 1, count << 4)
    end
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
        end
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        player:set_quest_var(EVENT_ID, 1, 0)
        player:event_notice(EVENT_ID, NOTICE_TRADED, {})
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
