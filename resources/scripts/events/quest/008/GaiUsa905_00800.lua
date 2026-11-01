-- Quest: Secret of the White Lily
--
-- Written from the client's quest script (kawari-questdump, 2026-11-01) and checked against Bob's Chronofoil capture
-- 412ba91d (2026-11-01, the whole quest: its fixture is tools/questkit/fixtures/800.json):
-- Sequence 0:   ACTOR0 (Aethelmaer) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1, with ITEM0 (UI8BH = 1; the client lists it from there.)
-- Sequences 1-3: ITEM0 shown to ACTOR1 (Mother Miounne), ACTOR2 (Bernadette) and ACTOR3 (Ceinguled) in turn: each
--               her or his NpcTrade in event mode (scene 2, 4, 6: a yield, 1 when handed over), then a talk (3, 5, 7):
--               notice 0, 1, 2 and the next sequence (255 after the third). ITEM0 stays (retail kept UI8BH = 1.)
-- Sequence 255: ACTOR4 (Ursandel) takes it (scene 8, NpcTrade, a yield) and rewards it (scene 9, a yield: QuestReward,
--               1 and the optional item chosen.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_TRADE = 8
SCENE_REWARD = 9

-- Those ITEM0 is shown to, in turn: their sequence, NpcTrade's scene, the talk after it and its notice.
SHOWINGS = {
    { sequence = 1, actor = ACTOR1, scene = 2, talk = 3, notice = 0, next = 2 },
    { sequence = 2, actor = ACTOR2, scene = 4, talk = 5, notice = 1, next = 3 },
    { sequence = 3, actor = ACTOR3, scene = 6, talk = 7, notice = 2, next = 255 },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_showing = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_showing = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 255 and BASE_ID == ACTOR4 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
        return
    end
    for _, showing in ipairs(SHOWINGS) do
        if sequence == showing.sequence and BASE_ID == showing.actor then
            current_showing = showing
            play(player, showing.scene, HIDE_HOTBAR)
            return
        end
    end
    player:finish_event()
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local showing = current_showing

    if scene == SCENE_OFFER then
        if results[1] == 1 then
            play(player, SCENE_ACCEPTED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 1)
    elseif showing ~= nil and scene == showing.scene then
        -- Shown: on to the talk.
        if results[1] == 1 then
            play(player, showing.talk, HIDE_HOTBAR)
            return
        end
        current_showing = nil
    elseif showing ~= nil and scene == showing.talk then
        current_showing = nil
        player:event_notice(EVENT_ID, showing.notice, {})
        player:quest_sequence(EVENT_ID, showing.next)
    elseif scene == SCENE_TRADE then
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
