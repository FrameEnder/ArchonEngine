-- Quest: Dance Dance Diplomacy
--
-- Written from the client's quest script (kawari-questdump, 2026-10-30) and checked against Bob's Chronofoil capture
-- 5fcc970c (2026-10-30, the whole quest: its fixture is tools/questkit/fixtures/3857.json):
-- Sequence 0:   ACTOR0 (Yda) offers it (scene 0, a yield: QuestOffer, 1 when accepted; the client turns it down
--               itself while QST_ACCEPT_CHECK, the old Dance Dance Diplomacy (717), is accepted), then scene 1, where
--               the client accepts it: sequence 1.
-- Sequence 1:   three sylphs to dance for, in any order (the client offers each while UI8AL < 3 and its flag is
--               unset); each has three scenes: talking (the first), the emote that wins them (the second) and the
--               third, taken here for any other emote (retail only danced):
--                 ACTOR1 (Ameexia), scenes 2, 3, 4, BitFlag8 1
--                 ACTOR2 (Knolexia), scenes 5, 6, 7, BitFlag8 2
--                 ACTOR3 (Dellexia), scenes 8, 9, 10, BitFlag8 3
--               Retail's /dance (Emote 11) on each played its second scene (0x2001): UI8AL counts them, notice 0
--               [count, 3]; at the third, the variables cleared and sequence 255.
-- Sequence 255: ACTOR0 rewards it (scene 11, a yield: QuestReward, 1; gil only). Retail then completed the old Dance
--               Dance Diplomacy (717) too: FinishQuest 3857, then 717.

SCENE_OFFER = 0
SCENE_ACCEPTED = 1
SCENE_REWARD = 11

EMOTE_FLAGS = HIDE_HOTBAR | NO_DEFAULT_CAMERA -- 0x2001 on retail
-- The emote retail used on each (Emote 11, /dance; Bob's Chronofoil capture 5fcc970c.)
EMOTE_DANCE = 11
NOTICE_DANCED = 0
SYLPHS_NEEDED = 3
-- The quest this one replaced, completed with it (retail's FinishQuest 717 right after 3857's.)
QUEST_ALSO_COMPLETED = 717

SYLPHS = {
    { actor = ACTOR1, talk = 2, danced = 3, other = 4, flag = 1 },
    { actor = ACTOR2, talk = 5, danced = 6, other = 7, flag = 2 },
    { actor = ACTOR3, talk = 8, danced = 9, other = 10, flag = 3 },
}

-- BitFlag8: flag 1 is the top bit of UI8F, as retail sets it.
function hasFlag(player, flag)
    return (player:get_quest_var(EVENT_ID, 5) & (0x80 >> (flag - 1))) ~= 0
end

function setFlag(player, flag)
    player:set_quest_var(EVENT_ID, 5, player:get_quest_var(EVENT_ID, 5) | (0x80 >> (flag - 1)))
end

-- The sylph `BASE_ID` is, if she's still to be won over.
function sylph(player)
    for _, sylph in ipairs(SYLPHS) do
        if BASE_ID == sylph.actor and not hasFlag(player, sylph.flag) then
            return sylph
        end
    end
    return nil
end

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
current_sylph = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)
    current_sylph = nil

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
        return
    elseif sequence == 1 then
        local found = sylph(player)
        if found ~= nil then
            play(player, found.talk, HIDE_HOTBAR)
            return
        end
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        play(player, SCENE_REWARD, HIDE_HOTBAR)
        return
    end
    player:finish_event()
end

function onEmote(emote, target, player)
    current_sylph = nil
    local found = player:get_quest_sequence(EVENT_ID) == 1 and sylph(player) or nil
    if found == nil then
        player:finish_event()
    elseif emote == EMOTE_DANCE then
        current_sylph = found
        play(player, found.danced, EMOTE_FLAGS)
    else
        play(player, found.other, EMOTE_FLAGS)
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
    elseif current_sylph ~= nil and scene == current_sylph.danced then
        local danced = (player:get_quest_var(EVENT_ID, 0) & 0x0F) + 1 -- UI8AL
        player:event_notice(EVENT_ID, NOTICE_DANCED, { danced, SYLPHS_NEEDED })
        if danced >= SYLPHS_NEEDED then
            for byte = 0, 5 do
                player:set_quest_var(EVENT_ID, byte, 0)
            end
            player:quest_sequence(EVENT_ID, 255)
        else
            player:set_quest_var(EVENT_ID, 0, (player:get_quest_var(EVENT_ID, 0) & 0xF0) | danced)
            setFlag(player, current_sylph.flag)
        end
        current_sylph = nil
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
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
