-- Quest: Reclamation (main scenario)
--
-- Written from the client's quest script (kawari-questdump, 2026-11-08) and checked against Bob's three Chronofoil
-- captures of 2026-11-08, each the whole quest (6a5244ec, 09ecf8e7 and 6c495fc4; fixtures
-- tools/questkit/fixtures/520.json, 520-09ecf8e7.json, 520-6c495fc4.json):
-- Sequence 0:   ACTOR0 (Alphinaud, at Ul'dah's airship landing) offers it (scene 0, a yield: QuestOffer, 1 when
--               accepted), then talks (scene 1, which ends with QuestAccepted): sequence 255.
-- Sequence 255: ACTOR2 (Alphinaud, in the Waking Sands) rewards it (scene 3, a yield: QuestReward, 1 and the item
--               picked; closing the window leaves UI8A 0x11); retail set UI8A to 0x11 as the cutscenes began (scene 4,
--               0x402400, its parameter the item picked: CUT_MANFST40510-40540, ending with QuestCompleted), and as
--               they ended the quest completed and the player went through a zone change within the Waking Sands
--               (all three runs): POPRANGE1, the parameter nothing else uses, is taken to be where. ACTOR3 (Cid) has a
--               line meanwhile (scene 5.) ACTOR1 (Elyenora, Ul'dah's airship ticketer, at the gate below the landing)
--               asks whether to go up (scene 2, a yield: YesNo): "yes" takes the player to POPRANGE0 in TERRITORYTYPE0
--               (4103351, Ul'dah's airship landing, where The Ul'dahn Envoy's ticketer puts you). Asked before it's
--               accepted too, so someone who left the landing gets back to Alphinaud there (unseen on retail; Bob's
--               bundle 20261111-101422: Lady of the Vortex's ticketer didn't.)

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_ELYENORA = 2
SCENE_REWARD = 3
SCENE_CUTSCENES = 4
SCENE_CID_LINE = 5

CUTSCENE_FLAGS = 0x402400 -- retail's

-- What retail sets UI8A to when a reward window is closed, and as the cutscenes after the reward began.
REWARD_CLOSED = 0x11

LINES = {
    [255] = { [ACTOR3] = SCENE_CID_LINE },
}

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
chosen = 0

function play(player, scene, flags, params)
    current_scene = scene
    player:play_scene(scene, flags, params or {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR2 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
    elseif (sequence == nil or sequence == 255) and BASE_ID == ACTOR1 then
        play(player, SCENE_ELYENORA, HIDE_HOTBAR)
    elseif sequence ~= nil and LINES[sequence] ~= nil and LINES[sequence][BASE_ID] ~= nil then
        play(player, LINES[sequence][BASE_ID], HIDE_HOTBAR)
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
        player:quest_sequence(EVENT_ID, 255)
    elseif scene == SCENE_ELYENORA then
        if results[1] == 1 then
            player:finish_event()
            player:change_territory_pop_range(TERRITORYTYPE0, POPRANGE0)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            chosen = results[2] or 0
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
            play(player, SCENE_CUTSCENES, CUTSCENE_FLAGS, { chosen })
            return
        elseif player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
        end
    elseif scene == SCENE_CUTSCENES then
        player:finish_event()
        player:finish_quest(EVENT_ID, chosen)
        -- A zone change within the Waking Sands, as retail's.
        player:move_to_pop_range(POPRANGE1, true)
        return
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
