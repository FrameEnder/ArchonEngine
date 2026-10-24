-- The Miner's and Botanist's actions (7.57 Action sheet, ClassJob 16 and 17), all run by GathererAction.lua.
-- Collectables' actions (Collect, Scour, Brazen, Meticulous, Scrutiny, Collector's Focus, Priming Touch) and Lay of
-- the Land aren't done yet.
local GATHERER_ACTIONS = {
    227, 210,              -- Prospect, Triangulate
    303, 304,              -- Sneak
    238, 221,              -- Truth of Mountains, Truth of Forests
    235, 218, 237, 220, 295, 294, -- Sharp Vision I-III, Field Mastery I-III
    4072, 4086,            -- Clear Vision, Flora Mastery
    21177, 21178, 25589, 25590, -- Mountaineer's and Pioneer's Gift I and II
    280, 282,              -- The Twelve's Bounty
    4073, 4087, 272, 273,  -- Bountiful Yield/Harvest I and II
    232, 215,              -- Solid Reason, Ageless Words
    239, 222, 241, 224,    -- King's Yield, Blessed Harvest I and II
    4081, 4095,            -- Luck of the Mountaineer, Luck of the Pioneer
    4589, 4590,            -- The Giving Land
    21203, 21204,          -- Nald'thal's Tidings, Nophica's Tidings
    26521, 26522,          -- Wise to the World
}

for _, action_id in ipairs(GATHERER_ACTIONS) do
    registerAction(action_id, BASE_DIR.."gathering/GathererAction.lua")
end
