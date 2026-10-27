-- Quest: Life, Materia and Everything (main scenario)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-27) and Bob's Chronofoil capture b8432613
-- (2026-10-27, the whole quest; its fixture is tools/questkit/fixtures/272.json).
-- Sequence 0:   ACTOR0 (Minfilia, in the Waking Sands) offers it (scene 0, a yield returning 1 when accepted; the client
--               accepts it in it): sequence 1 with ITEM0 (UI8BH 1; the client lists it as a key item from that).
-- Sequence 1:   ACTOR1 (Mutamix Bubblypots, in Ul'dah) takes ITEM0 (scene 1, an NpcTrade, a yield: 1 when handed
--               over), then the cutscene (scene 2, CUT_MANFST20810, retail's flags 0xF8402EFB): notice 0 [ITEM1's icon],
--               sequence 255 with ITEM1 in place of ITEM0 (UI8BL 1).
-- Sequence 255: ACTOR0 takes ITEM1 (scene 3, an NpcTrade, a yield) and rewards it (scene 4, a yield returning 1.)

SCENE_OFFER = 0
SCENE_MUTAMIX_TRADE = 1
SCENE_CUTSCENE = 2
SCENE_MINFILIA_TRADE = 3
SCENE_REWARD = 4

-- Retail's flags and the icon it sent with notice 0 (Bob's Chronofoil capture b8432613.)
CUTSCENE_FLAGS = 0xF8402EFB
ITEM1_ICON = 30412

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
        play(player, SCENE_MUTAMIX_TRADE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_MINFILIA_TRADE, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 1)
            player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0
        end
    elseif scene == SCENE_MUTAMIX_TRADE then
        -- 1 when the item was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_CUTSCENE, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_CUTSCENE then
        player:event_notice(EVENT_ID, 0, { ITEM1_ICON })
        player:quest_sequence(EVENT_ID, 255)
        player:set_quest_var(EVENT_ID, 1, 0x01) -- UI8BL: ITEM1, ITEM0 handed over
    elseif scene == SCENE_MINFILIA_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
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
