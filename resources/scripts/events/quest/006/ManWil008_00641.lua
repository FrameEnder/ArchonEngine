-- Quest: A Matter of Tradition (Ul'dah)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 641):
-- Sequence 0:   ACTOR0 (Owyne, in TERRITORYTYPE0 210) offers it (scene 0, a yield returning 1 when accepted; the
--               client offers her while UI8AL and BitFlag8 1 are unset), then scene 1, after which it's accepted:
--               sequence 1. ACTOR1 (the Sultansworn elite outside) asks whether to go in (scene 2, a yes/no: 1 for
--               yes), which takes the player back to POPRANGE0 in TERRITORYTYPE0 to accept it (retail accepted it
--               without leaving; without this, leaving the room first made the quest impossible to pick up.)
-- Sequence 1:   ACTOR2 (Momodi) (scene 3, while UI8AL and BitFlag8 1 are unset): notice 0, sequence 2. ACTOR0 only
--               talks (scene 4.)
-- Sequence 2:   ACTOR3 (Serendipity, the Steps of Thal) gives ITEM0 (scene 5): notice 1 with its icon (21002 on
--               retail), UI8BH = 1 (the client lists ITEM0 from UI8BH), sequence 255.
-- Sequence 255: ACTOR2 takes ITEM0 (scene 6, a hand-over window, a yield returning 1 when handed over), then the
--               reward (scene 7, a yield: QuestReward, 1; anything else cancels the trade.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_GO_IN = 2
SCENE_MOMODI = 3
SCENE_OWYNE = 4
SCENE_SERENDIPITY = 5
SCENE_HAND_OVER = 6
SCENE_FINISH = 7

NOTICE_MOMODI = 0
NOTICE_ITEM = 1
ITEM0_ICON = 21002 -- what retail's notice carried

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
    elseif sequence == nil and BASE_ID == ACTOR1 then
        play(player, SCENE_GO_IN, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR2 then
        play(player, SCENE_MOMODI, HIDE_HOTBAR)
    elseif sequence == 1 and BASE_ID == ACTOR0 then
        play(player, SCENE_OWYNE, HIDE_HOTBAR)
    elseif sequence == 2 and BASE_ID == ACTOR3 then
        play(player, SCENE_SERENDIPITY, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_HAND_OVER, HIDE_HOTBAR)
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
    elseif scene == SCENE_GO_IN then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
    elseif scene == SCENE_ACCEPTED then
        player:accept_quest(EVENT_ID)
        player:quest_sequence(EVENT_ID, 1)
    elseif scene == SCENE_MOMODI then
        player:event_notice(EVENT_ID, NOTICE_MOMODI)
        player:quest_sequence(EVENT_ID, 2)
    elseif scene == SCENE_SERENDIPITY then
        player:event_notice(EVENT_ID, NOTICE_ITEM, { ITEM0_ICON })
        player:set_quest_var(EVENT_ID, 1, 0x10) -- UI8BH = 1: ITEM0
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_HAND_OVER then
        -- 1 means ITEM0 was handed over.
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
