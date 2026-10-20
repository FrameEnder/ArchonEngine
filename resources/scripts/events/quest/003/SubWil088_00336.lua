-- Quest: Oh Captain, My Captain (Ul'dah, Western Thanalan)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-20) and retail (Bob's Chronofoil capture of
-- 2026-10-18, questkit replay 336):
-- Sequence 0:   ACTOR0 (Merilda) offers it (scene 0, a yield: the client's QuestOffer, 1 when accepted; the client
--               itself refuses while QST_ACCEPT_CHECK is accepted), then the cutscene (scene 1, 0xF8402EFB), where
--               the client accepts it; accepted after it, sequence 1.
-- Sequence 1:   ACTOR1 (Baldewyn, layout 3931797) sends you into QUESTBATTLE0 (scene 2, a yield: the client's
--               YesNoQuestBattle, 1 for yes; the client offers him while UI8AL and BitFlag8 1 are unset, which retail
--               never sets). Losing leaves the quest as it was (Bob lost twice and asked him again each time); winning
--               moves it to sequence 255 inside TERRITORYTYPE0 (questbattle/39.lua). ACTOR2..ACTOR6 and EOBJECT0 are
--               also offered in this sequence, but their scenes (3-8) are empty: the event just ends.
-- Sequence 255: ACTOR7 (Fufulupa) rewards it (scene 9, a yield: QuestReward, 1 and the chosen item; gil and fixed
--               items only on retail). No variables or notices at all.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ENTER_BATTLE = 2
SCENE_FINISH = 9

CUTSCENE_FLAGS = SET_BASE | HIDE_HOTBAR -- 0xF8402EFB

-- BitFlag8 n is bit 8 - n of UI8F.
function hasBitFlag8(player, n)
    return player:get_quest_var(EVENT_ID, 5) & (1 << (8 - n)) ~= 0
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
    elseif sequence == 1 and BASE_ID == ACTOR1 and player:get_quest_var(EVENT_ID, 0) & 0x0F == 0
        and not hasBitFlag8(player, 1) then
        play(player, SCENE_ENTER_BATTLE, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR7 then
        play(player, SCENE_FINISH, HIDE_HOTBAR)
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
    elseif scene == SCENE_ENTER_BATTLE then
        if results[1] == 1 then
            player:finish_event()
            player:join_quest_battle(QUESTBATTLE0, TERRITORYTYPE0)
            return
        end
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
