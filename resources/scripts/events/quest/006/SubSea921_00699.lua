-- Quest: Color Your World (Vesper Bay)
--
-- Checked against the client's quest script (kawari-questdump, 2026-10-25), Sapphire's script and retail (Bob's
-- Chronofoil capture of 2026-10-25, afe32479, questkit replay 699):
-- Sequence 0:   ACTOR0 (Swyrgeim) offers it (scene 0, a yield: QuestOffer, 1 when accepted); accepting goes straight
--               to sequence 255.
-- Sequence 255: with RITEM0 in the bags, ACTOR0 asks for it (scene 1: NpcTrade, a yield, 1 once it's handed over;
--               the client moves it into the trade window, HandIn, and leaves it in its bag), then rewards it
--               (scene 2, a yield: QuestReward, 1 and the chosen item; it also shows dyeing's how-to.) Retail took
--               the handed-in item from its bag only then, with the reward. Without it, ACTOR0 says so (scene 3:
--               the client's GetNumOfItems check, Sapphire's choice of scene.)
-- Completing it unlocks dyeing (UnlockLink 14, the Quest sheet's; the server's finish_quest sends it.)

SCENE_OFFER = 0
SCENE_TRADE = 1
SCENE_REWARD = 2
SCENE_NO_ITEM = 3

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil

function play(player, scene, flags)
    current_scene = scene
    player:play_scene(scene, flags, {})
end

-- Whether RITEM0 is in the bags. If the inventory can't be read, assume it is, so the quest can't get stuck.
function hasItem(player)
    local ok, count = pcall(function()
        return player.inventory:count_in_bags(RITEM0)
    end)
    return not ok or count == nil or count > 0
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil and BASE_ID == ACTOR0 then
        play(player, SCENE_OFFER, HIDE_HOTBAR)
    elseif sequence == 255 and BASE_ID == ACTOR0 then
        if hasItem(player) then
            play(player, SCENE_TRADE, HIDE_HOTBAR)
        else
            play(player, SCENE_NO_ITEM, HIDE_HOTBAR)
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
            player:accept_quest(EVENT_ID)
            player:quest_sequence(EVENT_ID, 255)
        end
    elseif scene == SCENE_TRADE then
        -- 1 when the item was handed over, nothing when the trade window was closed.
        if results[1] == 1 then
            play(player, SCENE_REWARD, HIDE_HOTBAR)
            return
        end
    elseif scene == SCENE_REWARD then
        if results[1] == 1 then
            player:take_handed_in_items()
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
