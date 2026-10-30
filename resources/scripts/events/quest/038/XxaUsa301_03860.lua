-- Quest: Never Forget
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- caa5e908 (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/3860.json):
-- Sequence 0:   ACTOR0 (Buscarron) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Never Forget (733), is accepted), then scene 1, where the
--               client accepts it, with a key item in UI8BL (the client lists the items from UI8BH and UI8BL):
--               sequence 1.
-- Sequence 1:   ACTOR1 (Baensyng) talks (scene 2) and gives the second (UI8BH = 1): notice 0 with its icon, sequence 2.
-- Sequence 2:   ACTOR2 (Kyokyoroon) takes the first (scene 3, NpcTrade in event mode: a yield, 1 when handed over;
--               UI8BL = 0), then scene 4: notice 1, sequence 3.
-- Sequence 3:   ACTOR3 (Wineburg) talks (scene 5): notice 2, sequence 4.
-- Sequence 4:   ACTOR4 (Ahldfoet) talks (scene 6): notice 3, sequence 255. ACTOR3 has a line from here on (scene 7.)
-- Sequence 255: ACTOR5 (Teteroon) takes the second (scene 8, NpcTrade) and rewards it (scene 9, a yield: QuestReward,
--               1 and the chosen item.) Retail then completed the old Never Forget (733) too: FinishQuest 3860, then
--               733.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_BAENSYNG = 2
SCENE_TRADE_KYOKYOROON = 3
SCENE_KYOKYOROON = 4
SCENE_WINEBURG = 5
SCENE_AHLDFOET = 6
SCENE_WINEBURG_AFTER = 7
SCENE_TRADE_TETEROON = 8
SCENE_REWARD = 9

NOTICE_BAENSYNG = 0
NOTICE_KYOKYOROON = 1
NOTICE_WINEBURG = 2
NOTICE_AHLDFOET = 3
-- The key item Baensyng gives: its icon, as retail's notice carried it (Bob's Chronofoil capture caa5e908.)
ITEM_ICON = 25252
-- The quest this one replaced, completed with it (retail's FinishQuest 733 right after 3860's.)
QUEST_ALSO_COMPLETED = 733

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
        play(player, SCENE_BAENSYNG, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR2 then
        play(player, SCENE_TRADE_KYOKYOROON, HIDE_HOTBAR)
    elseif sequence == 3 and BASE_ID == ACTOR3 then
        play(player, SCENE_WINEBURG, HIDE_HOTBAR)
    elseif sequence == 4 and BASE_ID == ACTOR4 then
        play(player, SCENE_AHLDFOET, HIDE_HOTBAR)
    elseif (sequence == 4 or sequence == 255) and BASE_ID == ACTOR3 then
        play(player, SCENE_WINEBURG_AFTER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR5 then
        play(player, SCENE_TRADE_TETEROON, HIDE_HOTBAR)
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
        setVar(player, 1, false, 1) -- UI8BL = 1
    elseif scene == SCENE_BAENSYNG then
        player:event_notice(EVENT_ID, NOTICE_BAENSYNG, { ITEM_ICON })
        setVar(player, 1, true, 1) -- UI8BH = 1
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_TRADE_KYOKYOROON then
        if results[1] == 1 then
            play(player, SCENE_KYOKYOROON, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_KYOKYOROON then
        player:event_notice(EVENT_ID, NOTICE_KYOKYOROON, {})
        setVar(player, 1, false, 0) -- handed over
        player:quest_sequence(EVENT_ID, 3)
    elseif scene == SCENE_WINEBURG then
        player:event_notice(EVENT_ID, NOTICE_WINEBURG, {})
        player:quest_sequence(EVENT_ID, 4)
    elseif scene == SCENE_AHLDFOET then
        player:event_notice(EVENT_ID, NOTICE_AHLDFOET, {})
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_TRADE_TETEROON then
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
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
