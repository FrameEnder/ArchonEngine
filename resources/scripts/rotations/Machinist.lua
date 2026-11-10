-- Machinist: its rotation, 1-100.
-- Priorities from Boss Mod's standard Machinist rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Machinist guide
-- and the official job guide (7.x): Split Shot - Slug Shot - Clean Shot (their Heated ones; Spread Shot / Scattergun on
-- three or more) for the Heat and Battery; Drill, Hot Shot / Air Anchor and Chain Saw (Excavator after it) on cooldown,
-- Reassemble before the strongest of them; Hypercharge at 50 Heat (or Hypercharged) when none of those is due, with
-- Wildfire before it, then Heat Blast / Blazing Shot (Auto Crossbow on three or more) through Overheated; Barrel
-- Stabilizer and Full Metal Field; the Rook Autoturret / Automaton Queen with the Battery; Gauss Round / Double Check and
-- Ricochet / Checkmate between. Retail's Machinists at 50-59 (two players, 1020 s) and 80-89 (one, 248 s): Heated Split,
-- Slug and Clean Shot 20, 20 and 17 uses, Gauss Round 12, Blazing Shot 12, Drill 7, Air Anchor 6, Reassemble 4,
-- Wildfire 3, Hypercharge 3.

local SPLIT, SLUG, CLEAN = 2866, 2868, 2873
local HEATED_SPLIT, HEATED_SLUG, HEATED_CLEAN = 7411, 7412, 7413
local SPREAD_SHOT, SCATTERGUN, AUTO_CROSSBOW = 2870, 25786, 16497
local HOT_SHOT, AIR_ANCHOR, DRILL, BIOBLASTER, CHAIN_SAW, EXCAVATOR, FULL_METAL_FIELD = 2872, 16500, 16498, 16499, 25788, 36981, 36982
local HEAT_BLAST, BLAZING_SHOT, HYPERCHARGE, WILDFIRE, REASSEMBLE, BARREL_STABILIZER = 7410, 36978, 17209, 2878, 2876, 7414
local GAUSS_ROUND, RICOCHET, DOUBLE_CHECK, CHECKMATE = 2874, 2890, 36979, 36980
local ROOK_AUTOTURRET, AUTOMATON_QUEEN = 2864, 16501

local S_OVERHEATED, S_REASSEMBLED, S_HYPERCHARGED, S_EXCAVATOR_READY, S_FULL_METAL = 2688, 851, 3864, 3865, 3866

-- The big weaponskills due within `seconds`.
local function tools_due(b, seconds)
    for _, action in ipairs({ DRILL, b:best(AIR_ANCHOR, HOT_SHOT), CHAIN_SAW }) do
        if action and b:has(action) and b:cd(action) < seconds then return true end
    end
    return b:has_status(S_EXCAVATOR_READY)
end

local function gcd(b)
    if b:has_status(S_OVERHEATED) then
        if b:aoe(3) and b:has(AUTO_CROSSBOW) then return AUTO_CROSSBOW end
        local blast = b:best(BLAZING_SHOT, HEAT_BLAST)
        if blast then return blast end
    end
    if b:has_status(S_FULL_METAL) and b:has(FULL_METAL_FIELD) then return FULL_METAL_FIELD end
    if b:has_status(S_EXCAVATOR_READY) and b:has(EXCAVATOR) then return EXCAVATOR end
    local battery = b:gauge_of("battery")
    local anchor = b:best(AIR_ANCHOR, HOT_SHOT)
    if anchor and b:ready(anchor) and battery <= 80 then return anchor end
    if b:ready(CHAIN_SAW) and battery <= 80 then return CHAIN_SAW end
    local drill = (b:aoe(3) and b:has(BIOBLASTER)) and BIOBLASTER or DRILL
    if b:ready(drill) then return drill end
    if b:aoe(3) then
        local area = b:best(SCATTERGUN, SPREAD_SHOT)
        if area then return area end
    end
    if b.combo == SLUG or b.combo == HEATED_SLUG then
        local finisher = b:best(HEATED_CLEAN, CLEAN)
        if finisher then return finisher end
    end
    if (b.combo == SPLIT or b.combo == HEATED_SPLIT) and b:has(SLUG) then return b:best(HEATED_SLUG, SLUG) end
    return b:best(HEATED_SPLIT, SPLIT)
end

local function ogcd(b)
    local heat, battery = b:gauge_of("heat"), b:gauge_of("battery")
    if b:ready(BARREL_STABILIZER) then return BARREL_STABILIZER, "self" end
    -- Reassemble for the next big weaponskill.
    if b:ready(REASSEMBLE) and not b:has_status(S_REASSEMBLED) and not b:has_status(S_OVERHEATED) and tools_due(b, 1.5) then
        return REASSEMBLE, "self"
    end
    local can_overheat = (heat >= 50 or b:has_status(S_HYPERCHARGED)) and not b:has_status(S_OVERHEATED)
        and not tools_due(b, 8)
    if can_overheat and b:ready(WILDFIRE) then return WILDFIRE end
    if can_overheat and b:ready(HYPERCHARGE) then return HYPERCHARGE, "self" end
    local pet = b:best(AUTOMATON_QUEEN, ROOK_AUTOTURRET)
    if pet and b:ready(pet) and battery >= 90 and b:gauge_of("pet_hits") == 0 then return pet, "self" end
    local gauss = b:best(DOUBLE_CHECK, GAUSS_ROUND)
    local ricochet = b:best(CHECKMATE, RICOCHET)
    if ricochet and b:ready(ricochet) and (not gauss or b:cd(ricochet) <= b:cd(gauss)) then return ricochet end
    if gauss and b:ready(gauss) then return gauss end
    return nil
end

return {
    classjobs = { 31 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
