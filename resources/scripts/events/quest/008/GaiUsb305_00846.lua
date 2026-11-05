-- Quest: That Weight
--
-- Written from the client's quest script (kawari-questdump, 2026-11-05) and checked against Bob's Chronofoil captures
-- b766138c (2026-11-05, the whole quest on a Gladiator: its fixture is tools/questkit/fixtures/846.json), 811b6f1b
-- (an Arcanist, `846-811b6f1b.json`) and 62c1dfe4 (a White Mage, `846-62c1dfe4.json`):
-- Sequence 0:   ACTOR0 (Drest) offers it (scene 0, a yield: QuestOffer, 1 when accepted), then scene 1, where the
--               client accepts it: sequence 1.
-- Sequence 1:   five of ENEMY0 (BNpcBase 136, tainted water sprites: Eastern La Noscea's own, not spawned for the
--               quest; any five, a different five each run) defeated, counted in UI8AL (the todo's): notice 0
--               [count, 5] with each; the fifth: UI8AL cleared, sequence 255. Other monsters killed on the way don't
--               count (Bob killed four between the third and fourth sprites, b766138c.)
-- Sequence 255: ACTOR0 rewards it (scene 2, a yield: QuestReward, 1 and the optional item chosen; closing the window
--               leaves it open, and retail then sets UI8A to 0x11, as in An Offer You Can Refuse.) Retail then
--               completed Not My War (847) too, with no rewards (FinishQuest 846, then 847, all three runs): Battle
--               Scars, next, is offered only while 847 isn't in the journal (its QST_ACCEPT_CHECK.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 2

NOTICE_DEFEATED = 0
SPRITES_NEEDED = 5
-- What retail set UI8A to when a reward window was closed (Bob's capture b766138c, quest 840; 414 and 680.)
REWARD_CLOSED = 0x11
-- Completed with this one, as retail did (Bob's Chronofoil captures of 2026-11-05.)
QUEST_ALSO_COMPLETED = 847

-- UI8AL: the sprites defeated.
function defeated(player)
    return player:get_quest_var(EVENT_ID, 0) & 0x0F
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
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onBNpcKill(base_id, layout_id, player)
    if player:get_quest_sequence(EVENT_ID) ~= 1 or base_id ~= ENEMY0 then
        return
    end
    local count = defeated(player) + 1
    player:event_notice(EVENT_ID, NOTICE_DEFEATED, { count, SPRITES_NEEDED })
    if count >= SPRITES_NEEDED then
        player:set_quest_var(EVENT_ID, 0, 0)
        player:quest_sequence(EVENT_ID, 255)
    else
        player:set_quest_var(EVENT_ID, 0, count)
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
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID, results[2])
            player:mark_quest_completed(QUEST_ALSO_COMPLETED)
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
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
