-- Viper: its rotation, 1-100.
-- Priorities from Boss Mod's standard Viper rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Viper guide and the
-- official job guide (7.x): Steel Fangs / Reaving Fangs as Honed Steel or Honed Reavers says, Hunter's Sting or
-- Swiftskin's Sting (the buff with less time left, or the one whose finisher the venom asks for), then the finisher the
-- venom names (Flanksting, Flanksbane, Hindsting, Hindsbane), Death Rattle after it; the area ones on three or more;
-- Vicewinder - Hunter's Coil - Swiftskin's Coil with their Twinfang and Twinblood Bites; Uncoiled Fury with the Rattling
-- Coils; Serpent's Ire, and Reawaken at 50 Serpent Offerings: the four Generations, their Legacies, Ouroboros. Retail's
-- Vipers at 80-89 (five players, 1292 s) used Death Rattle 125, Twinfang Bite 84, Twinblood Bite 84, Swiftskin's Sting
-- 69, Steel Fangs 68, Reaving Fangs 65, Hunter's Sting 61, Writhing Snap 47, Vicewinder 43, Hunter's Coil 43,
-- Swiftskin's Coil 42, each finisher 30-34.

local STEEL_FANGS, REAVING_FANGS, HUNTERS_STING, SWIFTSKINS_STING = 34606, 34607, 34608, 34609
local FLANKSTING, FLANKSBANE, HINDSTING, HINDSBANE = 34610, 34611, 34612, 34613
local STEEL_MAW, REAVING_MAW, HUNTERS_BITE, SWIFTSKINS_BITE, JAGGED_MAW, BLOODIED_MAW = 34614, 34615, 34616, 34617, 34618, 34619
local WRITHING_SNAP, DEATH_RATTLE, LAST_LASH = 34632, 34634, 34635
local VICEWINDER, HUNTERS_COIL, SWIFTSKINS_COIL, VICEPIT, HUNTERS_DEN, SWIFTSKINS_DEN = 34620, 34621, 34622, 34623, 34624, 34625
local TWINFANG_BITE, TWINBLOOD_BITE, TWINFANG_THRESH, TWINBLOOD_THRESH = 34636, 34637, 34638, 34639
local UNCOILED_FURY, UNCOILED_TWINFANG, UNCOILED_TWINBLOOD, SERPENTS_IRE = 34633, 34644, 34645, 34647
local REAWAKEN, OUROBOROS = 34626, 34631
local GENERATIONS = { 34627, 34628, 34629, 34630 }
local LEGACIES = { [34627] = 34640, [34628] = 34641, [34629] = 34642, [34630] = 34643 }

local S_INSTINCT, S_SWIFTSCALED, S_HONED_STEEL, S_HONED_REAVERS = 3668, 3669, 3672, 3772
local S_FLANKSTUNG, S_FLANKSBANE, S_HINDSTUNG, S_HINDSBANE, S_GRIMHUNTERS, S_GRIMSKINS = 3645, 3646, 3647, 3648, 3649, 3650
local S_HUNTERS_VENOM, S_SWIFTSKINS_VENOM, S_FELLHUNTERS, S_FELLSKINS = 3657, 3658, 3659, 3660
local S_POISED_TWINFANG, S_POISED_TWINBLOOD, S_REAWAKENED, S_READY_TO_REAWAKEN = 3665, 3666, 3670, 3671

local FINISHERS = { [FLANKSTING] = true, [FLANKSBANE] = true, [HINDSTING] = true, [HINDSBANE] = true }

local function gcd(b)
    if not b:in_melee() then
        if b:gauge_of("coils") > 0 and b:has(UNCOILED_FURY) then return UNCOILED_FURY end
        if b:has(WRITHING_SNAP) then return WRITHING_SNAP end
        return nil
    end
    local area = b.near_self >= 3
    -- Reawakened: the Generations in turn, Ouroboros at the last (from 96.)
    if b:has_status(S_REAWAKENED) then
        local tribute = b:gauge_of("tribute")
        local first = b.level >= 96 and 5 or 4
        if tribute == 1 and b.level >= 96 then return OUROBOROS end
        local step = first - tribute + 1
        if GENERATIONS[step] then return GENERATIONS[step] end
    end
    -- The coils' chain under way.
    if b.combo == VICEWINDER then return HUNTERS_COIL end
    if b.combo == HUNTERS_COIL then return SWIFTSKINS_COIL end
    if b.combo == VICEPIT then return HUNTERS_DEN end
    if b.combo == HUNTERS_DEN then return SWIFTSKINS_DEN end
    local between = b.combo == 0 or FINISHERS[b.combo] or b.combo == JAGGED_MAW or b.combo == BLOODIED_MAW
        or b.combo == SWIFTSKINS_COIL or b.combo == SWIFTSKINS_DEN or b.combo == OUROBOROS
    if between and b:has(REAWAKEN) and (b:gauge_of("offerings") >= 50 or b:has_status(S_READY_TO_REAWAKEN))
        and b:has_status(S_INSTINCT) and b:has_status(S_SWIFTSCALED) then
        return REAWAKEN
    end
    if between then
        local winder = area and b:best(VICEPIT) or b:best(VICEWINDER)
        if winder and b:ready(winder) and b:has_status(S_INSTINCT) then return winder end
        if b:gauge_of("coils") >= 3 and b:has(UNCOILED_FURY) then return UNCOILED_FURY end
    end
    if area and b:has(STEEL_MAW) then
        if b.combo == HUNTERS_BITE or b.combo == SWIFTSKINS_BITE then
            if b:has(JAGGED_MAW) then return b:has_status(S_GRIMSKINS) and BLOODIED_MAW or JAGGED_MAW end
        end
        if (b.combo == STEEL_MAW or b.combo == REAVING_MAW) and b:has(HUNTERS_BITE) then
            if b:has(SWIFTSKINS_BITE) and b:status(S_SWIFTSCALED) < b:status(S_INSTINCT) then return SWIFTSKINS_BITE end
            return HUNTERS_BITE
        end
        if b:has_status(S_HONED_REAVERS) and b:has(REAVING_MAW) then return REAVING_MAW end
        return STEEL_MAW
    end
    if b.combo == HUNTERS_STING and b:has(FLANKSTING) then
        return b:has_status(S_FLANKSBANE) and FLANKSBANE or FLANKSTING
    end
    if b.combo == SWIFTSKINS_STING and b:has(HINDSTING) then
        return b:has_status(S_HINDSTUNG) and HINDSTING or HINDSBANE
    end
    if (b.combo == STEEL_FANGS or b.combo == REAVING_FANGS) and b:has(HUNTERS_STING) then
        -- The sting whose finisher the venom names, else the buff with less time left.
        if b:has_status(S_HINDSTUNG) or b:has_status(S_HINDSBANE) then return b:best(SWIFTSKINS_STING, HUNTERS_STING) end
        if b:has_status(S_FLANKSTUNG) or b:has_status(S_FLANKSBANE) then return HUNTERS_STING end
        if b:has(SWIFTSKINS_STING) and b:status(S_SWIFTSCALED) < b:status(S_INSTINCT) then return SWIFTSKINS_STING end
        return HUNTERS_STING
    end
    if b:has_status(S_HONED_REAVERS) and b:has(REAVING_FANGS) then return REAVING_FANGS end
    return STEEL_FANGS
end

local function ogcd(b)
    if not b:in_melee() then return nil end
    if LEGACIES[b.combo] and b:ready(LEGACIES[b.combo]) then return LEGACIES[b.combo] end
    if b:has_status(S_HUNTERS_VENOM) and b:ready(TWINFANG_BITE) then return TWINFANG_BITE end
    if b:has_status(S_SWIFTSKINS_VENOM) and b:ready(TWINBLOOD_BITE) then return TWINBLOOD_BITE end
    if b:has_status(S_FELLHUNTERS) and b:ready(TWINFANG_THRESH) then return TWINFANG_THRESH, "self" end
    if b:has_status(S_FELLSKINS) and b:ready(TWINBLOOD_THRESH) then return TWINBLOOD_THRESH, "self" end
    if b:has_status(S_POISED_TWINFANG) and b:ready(UNCOILED_TWINFANG) then return UNCOILED_TWINFANG end
    if b:has_status(S_POISED_TWINBLOOD) and b:ready(UNCOILED_TWINBLOOD) then return UNCOILED_TWINBLOOD end
    if FINISHERS[b.combo] and b:ready(DEATH_RATTLE) then return DEATH_RATTLE end
    if (b.combo == JAGGED_MAW or b.combo == BLOODIED_MAW) and b:ready(LAST_LASH) then return LAST_LASH, "self" end
    if b:ready(SERPENTS_IRE) then return SERPENTS_IRE, "self" end
    return nil
end

return {
    classjobs = { 41 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
