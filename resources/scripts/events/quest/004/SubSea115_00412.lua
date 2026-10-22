-- Quest: Without a Doubt (Limsa Lominsa)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-21) and Bob's Chronofoil capture of the Limsa
-- story on retail (2026-10-21):
-- Sequence 0:   ACTOR0 offers it (scene 0, a yield: 1 accepts): accepted straight into sequence 255.
-- Sequence 255: ACTOR1 won't say (scene 1, SEQ_1_ACTOR1) until you use an emote on him (the Quest sheet listens for
--               one): the right one plays SEQ_1_ACTOR1_EMOTEOK (100, the reward: a yield, 1 completes it), any other
--               SEQ_1_ACTOR1_EMOTENO (99). Retail's right one was emote 12, the only one in the capture.

SCENE_OFFER = 0
SCENE_ACTOR1 = SEQ_1_ACTOR1 or 1
SCENE_EMOTE_OK = SEQ_1_ACTOR1_EMOTEOK or 100
SCENE_EMOTE_NO = SEQ_1_ACTOR1_EMOTENO or 99
-- The emote that does it (retail, Bob's capture; other emotes play SCENE_EMOTE_NO.)
EMOTE_OK = 12

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
    elseif sequence == 255 and BASE_ID == ACTOR1 then
        play(player, SCENE_ACTOR1, HIDE_HOTBAR)
    else
        player:finish_event()
    end
end

function onEmote(emote, target, player)
    if player:get_quest_sequence(EVENT_ID) ~= 255 or BASE_ID ~= ACTOR1 then
        player:finish_event()
    elseif emote == EMOTE_OK then
        play(player, SCENE_EMOTE_OK, HIDE_HOTBAR)
    else
        play(player, SCENE_EMOTE_NO, HIDE_HOTBAR)
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
        end
    elseif scene == SCENE_EMOTE_OK then
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
