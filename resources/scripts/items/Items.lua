ITEM_ACTION_TYPE_STATUS_REMOVAL = 842
ITEM_ACTION_TYPE_FOOD = 844
ITEM_ACTION_TYPE_FOOD_2 = 845
ITEM_ACTION_TYPE_FOOD_3 = 846
ITEM_ACTION_TYPE_HP_POTION = 847
ITEM_ACTION_TYPE_MP_POTION = 848
ITEM_ACTION_TYPE_ELIXIR = 849
ITEM_ACTION_TYPE_GYSHAL_GREENS = 851
ITEM_ACTION_TYPE_MINION = 853
ITEM_ACTION_TYPE_BUDDY_EQUIP = 1013
ITEM_ACTION_TYPE_MOUNT = 1322
ITEM_ACTION_TYPE_FANTASIA = 1326 -- this is also used elsewhere, not sure what
-- Items that teach an unlock link: emotes, hairstyles and many others.
ITEM_ACTION_TYPE_UNLOCK_LINK = 2633
ITEM_ACTION_TYPE_THAVNAIRIAN_ONION = 3292 -- the Thavnairian Onion's ItemAction (530)
ITEM_ACTION_TYPE_TRIPLE_TRIAD_CARD = 3357
ITEM_ACTION_TYPE_ORNAMENT = 20086
ITEM_ACTION_TYPE_ORCHESTRION = 25183
ITEM_ACTION_TYPE_FACEWEAR = 37312
-- Also an unlock link (the client checks it the same way as 2633.)
ITEM_ACTION_TYPE_UNLOCK_LINK_2 = 43141

-- Items that teach something, by ItemAction type: what they teach, and where its ID is (the ItemAction's Data[0], or
-- the item's AdditionalData). These are the types the client's own "already learned" check knows
-- (UIState::IsItemActionUnlocked), with the same IDs. Using one teaches it and uses the item up.
LEARNED_FROM_ITEMS = {
    [ITEM_ACTION_TYPE_MINION] = { kind = "minion", from = "data" },
    [ITEM_ACTION_TYPE_BUDDY_EQUIP] = { kind = "buddy_equip", from = "data" },
    [ITEM_ACTION_TYPE_MOUNT] = { kind = "mount", from = "data" },
    [ITEM_ACTION_TYPE_UNLOCK_LINK] = { kind = "unlock", from = "data" },
    [ITEM_ACTION_TYPE_UNLOCK_LINK_2] = { kind = "unlock", from = "data" },
    [ITEM_ACTION_TYPE_TRIPLE_TRIAD_CARD] = { kind = "triple_triad_card", from = "additional" },
    [ITEM_ACTION_TYPE_ORNAMENT] = { kind = "ornament", from = "data" },
    [ITEM_ACTION_TYPE_ORCHESTRION] = { kind = "orchestrion", from = "additional" },
    [ITEM_ACTION_TYPE_FACEWEAR] = { kind = "glasses", from = "additional" },
}

-- This is called whenever the client tries to use an item
function dispatchItem(player, id, action_type, action_data, additional_data, is_misc)
    local has_vfx = action_data[1] ~= 0
    local learned = LEARNED_FROM_ITEMS[action_type]
    if learned ~= nil then
        local learned_id = learned.from == "data" and action_data[1] or additional_data
        player:learn_from_item(learned.kind, learned_id, id)
        return runAction("items/GenericNoEffect.lua", 0)
    elseif action_type == ITEM_ACTION_TYPE_FANTASIA then
        return runAction("items/Fantasia.lua", 0)
    -- Combat items (ItemAction Data: action_data[1] is Data[0].)
    elseif action_type == ITEM_ACTION_TYPE_HP_POTION then
        return runAction("items/Potion.lua", action_data[1] * 65536 + action_data[2])
    elseif action_type == ITEM_ACTION_TYPE_MP_POTION then
        return runAction("items/Ether.lua", action_data[1] * 65536 + action_data[2])
    elseif action_type == ITEM_ACTION_TYPE_ELIXIR then
        return runAction("items/Elixir.lua", action_data[2] + action_data[4] * 65536)
    elseif action_type == ITEM_ACTION_TYPE_STATUS_REMOVAL then
        return runAction("items/StatusRemoval.lua", action_data[1])
    elseif action_type == ITEM_ACTION_TYPE_FOOD or action_type == ITEM_ACTION_TYPE_FOOD_2
        or action_type == ITEM_ACTION_TYPE_FOOD_3 then
        return runAction("items/Food.lua", action_data[2] + action_data[3] * 65536)
    elseif action_type == ITEM_ACTION_TYPE_GYSHAL_GREENS then
        return runAction("items/GyshalGreens.lua", additional_data)
    elseif action_type == ITEM_ACTION_TYPE_THAVNAIRIAN_ONION then
        player:feed_companion_onion(id)
        return runAction("items/GenericNoEffect.lua", 0)
    -- Otherwise, check if our item belongs to the Seasonal Miscellany or Miscellany item categories and has a vfx or not. Examples of ones that do play vfx: DAM, peach confetti. Examples of ones that don't play vfx: Realm Reborn Red, Heavenscracker.
    -- TODO: This may not be the best way but this seems to work for now
    elseif is_misc and has_vfx then
        return runAction("items/GenericVfx.lua", action_data[1])
    elseif is_misc and not has_vfx then
        return runAction("items/GenericNoEffect.lua", 0)
    else
        player:send_message("Unhandled item type: "..action_type.." (item id: "..id..")")
    end

    return nil
end
