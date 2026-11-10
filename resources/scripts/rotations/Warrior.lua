-- Marauder and Warrior: their rotation, 1-100.
-- Priorities from Boss Mod's standard Warrior rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Warrior guide
-- (7.x): keep Surging Tempest up (Storm's Eye when it has under ~10 s, else Storm's Path); spend the Beast Gauge on
-- Fell Cleave / Inner Beast (Decimate / Steel Cyclone on three or more) before it overcaps and freely under Inner
-- Release; Infuriate when it won't overcap; Inner Release / Berserk, Upheaval and Onslaught inside Surging Tempest;
-- Primal Rend, Primal Wrath and Primal Ruination when ready. Retail's Marauders at 20-29 (two players, 832 s) used Heavy
-- Swing 6.5 a minute, Maim 5.8, Overpower 5.6 (trash), Tomahawk 4.3, Berserk 0.9; its Warriors at 30-39 Overpower 9.5,
-- Heavy Swing 3.9, Maim 3.5, Storm's Path 3.0, Tomahawk 2.9, Berserk 0.8.

local HEAVY_SWING, MAIM, STORMS_PATH, STORMS_EYE = 31, 37, 42, 45
local OVERPOWER, MYTHRIL_TEMPEST = 41, 16462
local TOMAHAWK = 46
local INNER_BEAST, FELL_CLEAVE, INNER_CHAOS = 49, 3549, 16465
local STEEL_CYCLONE, DECIMATE, CHAOTIC_CYCLONE = 51, 3550, 16463
local BERSERK, INNER_RELEASE, INFURIATE = 38, 7389, 52
local UPHEAVAL, OROGENY, ONSLAUGHT = 7387, 25752, 7386
local PRIMAL_REND, PRIMAL_WRATH, PRIMAL_RUINATION = 25753, 36924, 36925

local S_SURGING_TEMPEST, S_BERSERK, S_INNER_RELEASE, S_NASCENT_CHAOS = 2677, 86, 1177, 1897
local S_PRIMAL_REND_READY, S_WRATHFUL, S_PRIMAL_RUINATION_READY = 2624, 3901, 3834

local function spender(b)
    if b:aoe(3) then
        if b:has_status(S_NASCENT_CHAOS) and b:has(CHAOTIC_CYCLONE) then return CHAOTIC_CYCLONE end
        local cyclone = b:best(DECIMATE, STEEL_CYCLONE)
        if cyclone then return cyclone end
    end
    if b:has_status(S_NASCENT_CHAOS) and b:has(INNER_CHAOS) then return INNER_CHAOS end
    return b:best(FELL_CLEAVE, INNER_BEAST)
end

local function gcd(b)
    local beast = b:gauge_of("beast")
    local tempest = b:status(S_SURGING_TEMPEST)
    if not b:in_melee() and b:has(TOMAHAWK) then return TOMAHAWK end
    if b:has_status(S_PRIMAL_RUINATION_READY) and b:has(PRIMAL_RUINATION) then return PRIMAL_RUINATION end
    if b:has_status(S_PRIMAL_REND_READY) and b:has(PRIMAL_REND) and b:status(S_INNER_RELEASE) == 0 then
        return PRIMAL_REND
    end
    -- Free (Inner Release) or a full enough gauge.
    local spend = spender(b)
    if spend and (b:has_status(S_INNER_RELEASE) or b:has_status(S_NASCENT_CHAOS) or beast >= 90
        or (beast >= 50 and (tempest > 0 or not b:has(STORMS_EYE)))) then
        return spend
    end
    if b:aoe(3) and b:has(OVERPOWER) then
        if b.combo == OVERPOWER and b:has(MYTHRIL_TEMPEST) then return MYTHRIL_TEMPEST end
        return OVERPOWER
    end
    if b.combo == MAIM then
        if b:has(STORMS_EYE) and tempest < 10 then return STORMS_EYE end
        if b:has(STORMS_PATH) then return STORMS_PATH end
    end
    if b.combo == HEAVY_SWING and b:has(MAIM) then return MAIM end
    return HEAVY_SWING
end

local function ogcd(b)
    local beast = b:gauge_of("beast")
    local tempest = b:status(S_SURGING_TEMPEST)
    local release = b:best(INNER_RELEASE, BERSERK)
    if release and b:ready(release) and b:in_melee() and (tempest > 0 or not b:has(STORMS_EYE)) then
        return release, "self"
    end
    if b:has_status(S_WRATHFUL) and b:ready(PRIMAL_WRATH) then return PRIMAL_WRATH end
    if b:ready(INFURIATE) and beast <= 50 and not b:has_status(S_INNER_RELEASE) and not b:has_status(S_NASCENT_CHAOS) then
        return INFURIATE, "self"
    end
    if tempest > 0 or not b:has(STORMS_EYE) then
        local up = (b:aoe(3) and b:has(OROGENY)) and OROGENY or UPHEAVAL
        if b:ready(up) and b:in_melee() then return up end
        if b:ready(ONSLAUGHT) and b:in_melee() then return ONSLAUGHT end
    end
    return nil
end

return {
    classjobs = { 3, 21 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
