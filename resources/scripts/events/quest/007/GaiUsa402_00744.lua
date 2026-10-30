-- Quest: Believe in Your Sylph
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30), Sapphire's GaiUsa402 and checked against
-- Bob's Chronofoil capture 9df4ff01 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/744.json):
-- Sequence 0:   ACTOR0 (Komuxio) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, a cutscene
--               (CUT_EVENT) where the client accepts it: sequence 1, with UI8BH = 1 (ITEM0 held.)
-- Sequence 1:   the lookout emote (22) in the quest's area (EVENTRANGE0; its listener is ACTOR1, 5010000, of type
--               Emote): the client asks for the event itself (StartEmoteEvent on the player's own actor) and retail
--               played the cutscene (scene 3, 0xF8402EFB): notice 0, sequence 255.
-- Sequence 255: ACTOR5 (Vorsaile Heuloix) takes ITEM0 (scene 7, NpcTrade in event mode: a yield, 1 when handed over)
--               and rewards it (scene 8, a yield: QuestReward, 1.)
-- The client's other scenes (2, 4-6, 9-11: ACTOR1, EOBJECT0 and ACTOR2-4 asked) are empty and aren't played.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_LOOKOUT = 3
SCENE_TRADE = 7
SCENE_REWARD = 8

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail

-- The emote the quest asks for (Sapphire's EmoteLookout; retail's StartEmoteEvent had 22.)
EMOTE_LOOKOUT = 22

NOTICE_LOOKOUT = 0

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
    elseif sequence == 255 and BASE_ID == ACTOR5 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    if player:get_quest_sequence(EVENT_ID) == 1 and emote == EMOTE_LOOKOUT then
        play(player, SCENE_LOOKOUT, CUTSCENE_FLAGS)
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
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH: ITEM0 held
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_LOOKOUT then
        player:event_notice(EVENT_ID, NOTICE_LOOKOUT, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
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
