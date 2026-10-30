-- Quest: First Contact
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/715.json):
-- Sequence 0:   ACTOR0 (Rolfe Hawthorne) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then the cutscene
--               (scene 90, 0xF8402EFB), where the client accepts it and ITEM0 is given (UI8BH = 1; the client lists
--               ITEM0 from UI8BH and ITEM1 from UI8BL in every sequence): sequence 1.
-- Sequence 1:   ACTOR1 (Amelain) talks (scene 2) and gives ITEM1: notice 0 with its icon, UI8BL = 1, sequence 2.
-- Sequence 2:   ACTOR2 (Komuxio) talks (scene 3): notice 1, sequence 3.
-- Sequence 3:   ACTOR2 takes an emote (the quest's emote listener for him in sequence 3): retail's /dance (Emote 11)
--               played scene 5 (0x2001, his joy): notice 2, sequence 255. The client's scene 6 is his goodbye, taken
--               here as the answer to any other emote (not seen on retail.) Talking to him plays scene 4.
-- Sequence 255: ACTOR2 takes the items (scene 7, NpcTrade in event mode: a yield, 1 when handed over), scene 8, then
--               the cutscene with the reward (scene 9, 0xF8402EFB, a yield: QuestReward, 1 and the chosen item.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 90
SCENE_AMELAIN = 2
SCENE_KOMUXIO = 3
SCENE_KOMUXIO_WAITS = 4
SCENE_EMOTE_RIGHT = 5
SCENE_EMOTE_WRONG = 6
SCENE_TRADE = 7
SCENE_TRADED = 8
SCENE_REWARD = 9

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB on retail
EMOTE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- The emote retail used on Komuxio (Emote 11, /dance; Bob's Chronofoil capture 5fcc970c.)
EMOTE_DANCE = 11

NOTICE_ITEM1 = 0
NOTICE_KOMUXIO = 1
NOTICE_EMOTE = 2
-- ITEM1's icon, as retail's notice carried it.
ITEM1_ICON = 25918

-- Quest variables: UI8A..F are bytes 0..5, H is the high nibble and L the low one.
function setVar(player, byte, high, nibble)
    local value = player:get_quest_var(EVENT_ID, byte)
    if high then
        value = (value & 0x0F) | (nibble << 4)
    else
        value = (value & 0xF0) | nibble
    end
    player:set_quest_var(EVENT_ID, byte, value)
end

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
        play(player, SCENE_AMELAIN, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_KOMUXIO, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR2 then
        play(player, SCENE_KOMUXIO_WAITS, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    if player:get_quest_sequence(EVENT_ID) == 3 and BASE_ID == ACTOR2 then
        if emote == EMOTE_DANCE then
            play(player, SCENE_EMOTE_RIGHT, EMOTE_FLAGS)
        else
            play(player, SCENE_EMOTE_WRONG, EMOTE_FLAGS)
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
            play(player, SCENE_ACCEPTED, CUTSCENE_FLAGS)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
        setVar(player, 1, true, 1) -- UI8BH = 1: ITEM0
    elseif scene == SCENE_AMELAIN then
        player:event_notice(EVENT_ID, NOTICE_ITEM1, { ITEM1_ICON })
        setVar(player, 1, false, 1) -- UI8BL = 1: ITEM1
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_KOMUXIO then
        player:event_notice(EVENT_ID, NOTICE_KOMUXIO, {})
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_EMOTE_RIGHT then
        player:event_notice(EVENT_ID, NOTICE_EMOTE, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE then
        if results[1] == 1 then
            play(player, SCENE_TRADED, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_TRADED then
        play(player, SCENE_REWARD, CUTSCENE_FLAGS)
        return
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
