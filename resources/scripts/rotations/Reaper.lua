-- Reaper: its rotation, 1-100.
-- Priorities from Boss Mod's standard Reaper rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Reaper guide and the
-- official job guide (7.x): Death's Design kept on the target (Shadow of Death, Whorl of Death on three or more); Slice -
-- Waxing Slice - Infernal Slice (Spinning Scythe - Nightmare Scythe) and Soul Slice (Soul Scythe) for the Soul, spent on
-- Gluttony and Blood Stalk / Unveiled Gibbet / Gallows (Grim Swathe), each followed by Gibbet / Gallows as their Enhanced
-- status says (Guillotine on three or more; the Executioner's ones after Gluttony from 96); Enshroud with 50 Shroud:
-- Void and Cross Reaping in turn, Lemure's Slice at two Void Shroud, Sacrificium, Communio at the last; Arcane Circle and
-- Plentiful Harvest, Perfectio. Retail's Reapers at 80-89 (six players, 1702 s) used Slice 113, Waxing Slice 101,
-- Infernal Slice 91, Gallows 70, Gibbet 64, Soul Slice 63, Shadow of Death 57, Cross Reaping 49, Void Reaping 43,
-- Unveiled Gibbet 43, Unveiled Gallows 42, Gluttony 28, Enshroud 21, Arcane Circle 17.

local SLICE, WAXING, INFERNAL, SPINNING, NIGHTMARE = 24373, 24374, 24375, 24376, 24377
local SHADOW_OF_DEATH, WHORL_OF_DEATH, HARPE, SOUL_SLICE, SOUL_SCYTHE = 24378, 24379, 24386, 24380, 24381
local GIBBET, GALLOWS, GUILLOTINE = 24382, 24383, 24384
local EX_GIBBET, EX_GALLOWS, EX_GUILLOTINE = 36970, 36971, 36972
local BLOOD_STALK, GRIM_SWATHE, UNVEILED_GIBBET, UNVEILED_GALLOWS, GLUTTONY = 24389, 24392, 24390, 24391, 24393
local ENSHROUD, VOID_REAPING, CROSS_REAPING, GRIM_REAPING, COMMUNIO = 24394, 24395, 24396, 24397, 24398
local LEMURES_SLICE, LEMURES_SCYTHE, SACRIFICIUM = 24399, 24400, 36969
local ARCANE_CIRCLE, PLENTIFUL_HARVEST, PERFECTIO, HARVEST_MOON = 24405, 24385, 36973, 24388

local S_DEATHS_DESIGN, S_SOUL_REAVER, S_EXECUTIONER = 2586, 2587, 3858
local S_ENHANCED_GIBBET, S_ENHANCED_GALLOWS, S_ENHANCED_VOID, S_ENHANCED_CROSS = 2588, 2589, 2590, 2591
local S_ENSHROUDED, S_IMMORTAL_SACRIFICE, S_BLOODSOWN, S_IDEAL_HOST, S_OBLATIO, S_PERFECTIO_PARATA, S_SOULSOW =
    2593, 2592, 2972, 3905, 3857, 3860, 2594

local function gcd(b)
    if not b:in_melee() then
        if b:has_status(S_SOULSOW) and b:has(HARVEST_MOON) then return HARVEST_MOON end
        if b:has(HARPE) then return HARPE end
        return nil
    end
    local area = b.near_self >= 3
    if b:has_status(S_ENSHROUDED) then
        local lemure = b:gauge_of("lemure")
        if lemure == 1 and b:has(COMMUNIO) then return COMMUNIO end
        if area and b:has(GRIM_REAPING) then return GRIM_REAPING end
        if b:has_status(S_ENHANCED_CROSS) then return CROSS_REAPING end
        return VOID_REAPING
    end
    if b:has_status(S_PERFECTIO_PARATA) and b:has(PERFECTIO) then return PERFECTIO end
    if b:has_status(S_IMMORTAL_SACRIFICE) and not b:has_status(S_BLOODSOWN) and b:has(PLENTIFUL_HARVEST) then
        return PLENTIFUL_HARVEST
    end
    local executioner = b:has_status(S_EXECUTIONER)
    if executioner or b:has_status(S_SOUL_REAVER) then
        if area then return executioner and EX_GUILLOTINE or GUILLOTINE end
        if b:has_status(S_ENHANCED_GALLOWS) then return executioner and EX_GALLOWS or GALLOWS end
        return executioner and EX_GIBBET or GIBBET
    end
    -- Death's Design kept up.
    if b:dot(S_DEATHS_DESIGN) < 5 and b.target_hp > 10 then
        if area and b:has(WHORL_OF_DEATH) then return WHORL_OF_DEATH end
        return SHADOW_OF_DEATH
    end
    if b:gauge_of("soul") <= 50 then
        local slice = (area and b:has(SOUL_SCYTHE)) and SOUL_SCYTHE or SOUL_SLICE
        if b:ready(slice) then return slice end
    end
    if area and b:has(SPINNING) then
        if b.combo == SPINNING and b:has(NIGHTMARE) then return NIGHTMARE end
        return SPINNING
    end
    if b.combo == WAXING and b:has(INFERNAL) then return INFERNAL end
    if b.combo == SLICE and b:has(WAXING) then return WAXING end
    return SLICE
end

local function ogcd(b)
    if not b:in_melee() then return nil end
    if b:has_status(S_ENSHROUDED) then
        if b:gauge_of("void") >= 2 then
            local lemure = b.near_self >= 3 and b:best(LEMURES_SCYTHE) or b:best(LEMURES_SLICE)
            if lemure and b:ready(lemure) then return lemure end
        end
        if b:has_status(S_OBLATIO) and b:ready(SACRIFICIUM) then return SACRIFICIUM end
        return nil
    end
    if b:ready(ARCANE_CIRCLE) then return ARCANE_CIRCLE, "self" end
    local reaving = b:has_status(S_SOUL_REAVER) or b:has_status(S_EXECUTIONER)
    if reaving then return nil end
    if b:ready(ENSHROUD) and (b:gauge_of("shroud") >= 50 or b:has_status(S_IDEAL_HOST)) then return ENSHROUD, "self" end
    local soul = b:gauge_of("soul")
    if soul >= 50 then
        if b:ready(GLUTTONY) then return GLUTTONY end
        -- Blood Stalk and its kind while Gluttony is a while off.
        if not b:has(GLUTTONY) or b:cd(GLUTTONY) > 5 or soul >= 100 then
            if b.near_self >= 3 and b:has(GRIM_SWATHE) then return GRIM_SWATHE end
            if b:has_status(S_ENHANCED_GALLOWS) and b:has(UNVEILED_GALLOWS) then return UNVEILED_GALLOWS end
            if b:has_status(S_ENHANCED_GIBBET) and b:has(UNVEILED_GIBBET) then return UNVEILED_GIBBET end
            if b:ready(BLOOD_STALK) then return BLOOD_STALK end
        end
    end
    return nil
end

return {
    classjobs = { 39 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
