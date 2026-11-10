-- Red Mage: its rotation, 1-100.
-- Priorities from Boss Mod's standard Red Mage rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Red Mage guide and
-- the official job guide (7.x): Dualcast's instant spell on Verthunder / Veraero (III) for the colour behind (Impact /
-- Scatter on three or more), a cast spell otherwise (Verfire / Verstone on their procs, else Jolt; Verthunder II /
-- Veraero II on three or more); at 50 of both Mana the Enchanted Riposte - Zwerchhau - Redoublement (Moulinets on three or
-- more) in melee or under Manafication's Magicked Swordplay, then Verflare / Verholy (the colour behind), Scorch,
-- Resolution; Grand Impact on its proc; Embolden, Manafication, Fleche, Contre Sixte, Acceleration, Vice of Thorns and
-- Prefulgence on cooldown; Enchanted Reprise when the Mana would overflow. Retail's Red Mages at 50-59 (four players,
-- 940 s) used Jolt 81, Verthunder 74, Veraero 70, Fleche 43, Verfire 36, Verstone 35, Engagement 21, Enchanted
-- Riposte 17, Zwerchhau 17, Redoublement 16, Scatter 15, Acceleration 12.

local JOLT, JOLT_II, JOLT_III = 7503, 7524, 37004
local VERTHUNDER, VERAERO, VERTHUNDER_III, VERAERO_III = 7505, 7507, 25855, 25856
local VERTHUNDER_II, VERAERO_II, SCATTER, IMPACT, GRAND_IMPACT = 16524, 16525, 7509, 16526, 37006
local VERFIRE, VERSTONE, VERFLARE, VERHOLY, SCORCH, RESOLUTION = 7510, 7511, 7525, 7526, 16530, 25858
local RIPOSTE, ZWERCHHAU, REDOUBLEMENT = 7527, 7528, 7529
local MOULINET, MOULINET_DEUX, MOULINET_TROIS, REPRISE = 7530, 37002, 37003, 16528
local FLECHE, CONTRE_SIXTE, EMBOLDEN, MANAFICATION, ACCELERATION, VICE_OF_THORNS, PREFULGENCE =
    7517, 7519, 7520, 7521, 7518, 37005, 37007

local S_VERFIRE_READY, S_VERSTONE_READY, S_ACCELERATION, S_DUALCAST, S_SWORDPLAY = 1234, 1235, 1238, 1249, 3875
local S_THORNED, S_GRAND_IMPACT_READY, S_PREFULGENCE_READY = 3876, 3877, 3878

local function gcd(b)
    local white, black = b:gauge_of("white_mana"), b:gauge_of("black_mana")
    local area = b:aoe(3)
    -- The melee combo under way, then the finishers.
    if b.combo == ZWERCHHAU and b:has(REDOUBLEMENT) then return REDOUBLEMENT end
    if b.combo == RIPOSTE and b:has(ZWERCHHAU) then return ZWERCHHAU end
    if b.combo == MOULINET_DEUX then return MOULINET_TROIS end
    if b.combo == MOULINET and b:has(MOULINET_DEUX) then return MOULINET_DEUX end
    if b.combo == SCORCH and b:has(RESOLUTION) then return RESOLUTION end
    if (b.combo == VERFLARE or b.combo == VERHOLY) and b:has(SCORCH) then return SCORCH end
    if b:gauge_of("mana_stacks") >= 3 and b:has(VERFLARE) then
        if b:has(VERHOLY) and white < black then return VERHOLY end
        return VERFLARE
    end
    if b:has_status(S_GRAND_IMPACT_READY) and b:has(GRAND_IMPACT) then return GRAND_IMPACT end
    -- The melee combo at 50 of both (in reach, or under Magicked Swordplay at any range.)
    local swordplay = b:has_status(S_SWORDPLAY)
    if (white >= 50 and black >= 50 or swordplay) and (b:in_melee() or swordplay) and b:has(RIPOSTE) then
        if area and b:has(MOULINET) then return MOULINET end
        return RIPOSTE
    end
    if white >= 95 and black >= 95 and b:has(REPRISE) then return REPRISE end
    local behind_white = white < black
    local instant = b:has_status(S_DUALCAST) or b:has_status(S_ACCELERATION)
    if instant then
        if area and b:has(SCATTER) then return b:best(IMPACT, SCATTER) end
        if behind_white then return b:best(VERAERO_III, VERAERO) or b:best(VERTHUNDER_III, VERTHUNDER) end
        return b:best(VERTHUNDER_III, VERTHUNDER)
    end
    if area and b:has(VERTHUNDER_II) then
        if behind_white and b:has(VERAERO_II) then return VERAERO_II end
        return VERTHUNDER_II
    end
    local fire = b:has_status(S_VERFIRE_READY) and b:has(VERFIRE)
    local stone = b:has_status(S_VERSTONE_READY) and b:has(VERSTONE)
    if stone and (behind_white or not fire) then return VERSTONE end
    if fire then return VERFIRE end
    return b:best(JOLT_III, JOLT_II, JOLT)
end

local function ogcd(b)
    if b:ready(EMBOLDEN) then return EMBOLDEN, "self" end
    if b:ready(MANAFICATION) and b:gauge_of("mana_stacks") == 0 and b.combo ~= RIPOSTE and b.combo ~= ZWERCHHAU then
        return MANAFICATION, "self"
    end
    if b:has_status(S_PREFULGENCE_READY) and b:ready(PREFULGENCE) then return PREFULGENCE end
    if b:has_status(S_THORNED) and b:ready(VICE_OF_THORNS) then return VICE_OF_THORNS end
    if b:ready(FLECHE) then return FLECHE end
    if b:ready(CONTRE_SIXTE) then return CONTRE_SIXTE end
    if b:ready(ACCELERATION) and not b:has_status(S_DUALCAST) and not b:has_status(S_ACCELERATION)
        and not b:has_status(S_VERFIRE_READY) and not b:has_status(S_VERSTONE_READY) then
        return ACCELERATION, "self"
    end
    return nil
end

return {
    classjobs = { 35 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
