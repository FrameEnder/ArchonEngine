-- Gladiator and Paladin: their rotation, 1-100.
-- Priorities from Boss Mod's standard Paladin rotation (xan, awgil/ffxiv_bossmod, BSD-3) and The Balance's Paladin
-- guide (7.x): Fight or Flight on cooldown with Requiescat/Imperator right after it; under Requiescat the Confiteor
-- chain; Goring Blade while it's ready; the Atonement chain (Supplication, Sepulchre) and Holy Spirit under Divine
-- Might in Fight or Flight or before they'd be lost; otherwise the Fast Blade - Riot Blade - Royal Authority combo, and
-- Total Eclipse - Prominence on three or more. Retail's Paladins at 30-39 in Bob's captures (three players, 624 s)
-- used Fast Blade, Riot Blade and Rage of Halone about five times a minute each, Total Eclipse about 7 (trash), Spirits
-- Within 1.7, Fight or Flight 1.1.

local FAST_BLADE, RIOT_BLADE, RAGE_OF_HALONE, ROYAL_AUTHORITY = 9, 15, 21, 3539
local TOTAL_ECLIPSE, PROMINENCE = 7381, 16457
local FIGHT_OR_FLIGHT, SPIRITS_WITHIN, EXPIACION, CIRCLE_OF_SCORN = 20, 29, 25747, 23
local SHIELD_LOB = 24
local GORING_BLADE, HOLY_SPIRIT, HOLY_CIRCLE = 3538, 7384, 16458
local REQUIESCAT, IMPERATOR = 7383, 36921
local ATONEMENT, SUPPLICATION, SEPULCHRE = 16460, 36918, 36919
local CONFITEOR, BLADE_OF_FAITH, BLADE_OF_TRUTH, BLADE_OF_VALOR, BLADE_OF_HONOR = 16459, 25748, 25749, 25750, 36922

local S_FIGHT_OR_FLIGHT, S_GORING_READY, S_DIVINE_MIGHT = 76, 3847, 2673
local S_ATONEMENT_READY, S_SUPPLICATION_READY, S_SEPULCHRE_READY = 1902, 3827, 3828
local S_REQUIESCAT, S_CONFITEOR_READY, S_BLADE_OF_HONOR_READY = 1368, 3019, 3831

local function gcd(b)
    local fof = b:status(S_FIGHT_OR_FLIGHT)
    if not b:in_melee() and b:has(SHIELD_LOB) and not b:has_status(S_DIVINE_MIGHT) then
        return SHIELD_LOB
    end
    -- The Confiteor chain under Requiescat (each step follows the last).
    if b:has_status(S_REQUIESCAT) and b.mp >= 1000 then
        if b.combo == BLADE_OF_TRUTH and b:has(BLADE_OF_VALOR) then return BLADE_OF_VALOR end
        if b.combo == BLADE_OF_FAITH and b:has(BLADE_OF_TRUTH) then return BLADE_OF_TRUTH end
        if b.combo == CONFITEOR and b:has(BLADE_OF_FAITH) then return BLADE_OF_FAITH end
        if b:has_status(S_CONFITEOR_READY) and b:has(CONFITEOR) then return CONFITEOR end
        -- Before Confiteor, Requiescat makes Holy Spirit instant and strong.
        if b:has(HOLY_SPIRIT) then
            return (b:aoe(3) and b:has(HOLY_CIRCLE)) and HOLY_CIRCLE or HOLY_SPIRIT
        end
    end
    if b:has_status(S_GORING_READY) and b:has(GORING_BLADE) then return GORING_BLADE end
    if b:has_status(S_SEPULCHRE_READY) and b:has(SEPULCHRE) then return SEPULCHRE end
    if b:has_status(S_SUPPLICATION_READY) and b:has(SUPPLICATION) then return SUPPLICATION end
    if b:has_status(S_DIVINE_MIGHT) and b:has(HOLY_SPIRIT) and (fof > 0 or b:status(S_DIVINE_MIGHT) < 6) then
        return (b:aoe(3) and b:has(HOLY_CIRCLE)) and HOLY_CIRCLE or HOLY_SPIRIT
    end
    if b:has_status(S_ATONEMENT_READY) and b:has(ATONEMENT) and (fof > 0 or b:status(S_ATONEMENT_READY) < 8) then
        return ATONEMENT
    end
    if b:aoe(3) and b:has(TOTAL_ECLIPSE) then
        if b.combo == TOTAL_ECLIPSE and b:has(PROMINENCE) then return PROMINENCE end
        return TOTAL_ECLIPSE
    end
    if b.combo == RIOT_BLADE then
        local finisher = b:best(ROYAL_AUTHORITY, RAGE_OF_HALONE)
        if finisher then return finisher end
    end
    if b.combo == FAST_BLADE and b:has(RIOT_BLADE) then return RIOT_BLADE end
    if b:has_status(S_ATONEMENT_READY) and b:has(ATONEMENT) then return ATONEMENT end
    if b:has_status(S_DIVINE_MIGHT) and b:has(HOLY_SPIRIT) then return HOLY_SPIRIT end
    return FAST_BLADE
end

local function ogcd(b)
    if b:ready(FIGHT_OR_FLIGHT) and b:in_melee() then return FIGHT_OR_FLIGHT, "self" end
    local fof = b:status(S_FIGHT_OR_FLIGHT)
    local req = b:best(IMPERATOR, REQUIESCAT)
    if req and b:ready(req) and (fof > 0 or not b:has(FIGHT_OR_FLIGHT) or b:cd(FIGHT_OR_FLIGHT) > 15) then return req end
    if b:has_status(S_BLADE_OF_HONOR_READY) and b:ready(BLADE_OF_HONOR) then return BLADE_OF_HONOR end
    local waiting = b:has(FIGHT_OR_FLIGHT) and b:cd(FIGHT_OR_FLIGHT) < 5
    if not waiting then
        if b:ready(CIRCLE_OF_SCORN) and b.near_self >= 1 then return CIRCLE_OF_SCORN, "self" end
        local sw = b:best(EXPIACION, SPIRITS_WITHIN)
        if sw and b:ready(sw) and b:in_melee() then return sw end
    end
    return nil
end

return {
    classjobs = { 1, 19 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
