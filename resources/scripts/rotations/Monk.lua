-- Pugilist and Monk: their rotation, 1-100.
-- Priorities from Boss Mod's standard Monk rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Monk guide (7.x):
-- the three forms in turn (Opo-opo, Raptor, Coeurl); in each, the action that builds its Fury when it has none
-- (Dragon Kick, Twin Snakes, Demolish) and otherwise the one that spends it (Bootshine / Leaping Opo, True Strike /
-- Rising Raptor, Snap Punch / Pouncing Coeurl), the area ones (Arm of the Destroyer, Four-point Fury, Rockbreaker) on
-- three or more; Perfect Balance for three actions of any form, which from 60 open the Beast Chakra: three of one
-- (Opo-opo) for the Lunar Nadi, three different for the Solar, then Masterful Blitz (Elixir Field / Burst, Flint Strike
-- / Rising Phoenix, Tornado Kick / Phantom Rush with both Nadi open); Fire's Reply and Wind's Reply after their
-- Riddles; Steel Peak / The Forbidden Chakra at five chakra; Riddle of Fire, Brotherhood and Riddle of Wind on
-- cooldown. The Furies are the server's gauge (opo_fury, raptor_fury, coeurl_fury). Retail's Monks at 50-59 (two
-- players, 556 s) used Dragon Kick 5.6 a minute, Bootshine 4.9, Snap Punch 3.2, True Strike 2.9, Twin Snakes 2.8,
-- Demolish 1.6.

local BOOTSHINE, TRUE_STRIKE, SNAP_PUNCH = 53, 54, 56
local DRAGON_KICK, TWIN_SNAKES, DEMOLISH = 74, 61, 66
local LEAPING_OPO, RISING_RAPTOR, POUNCING_COEURL = 36945, 36946, 36947
local ARM_OF_THE_DESTROYER, SHADOW_OF_THE_DESTROYER, FOUR_POINT_FURY, ROCKBREAKER = 62, 25767, 16473, 70
local STEEL_PEAK, FORBIDDEN_CHAKRA, HOWLING_FIST, ENLIGHTENMENT = 25761, 3547, 25763, 16474
local STEELED_MEDITATION, FORBIDDEN_MEDITATION = 36940, 36942
local PERFECT_BALANCE, RIDDLE_OF_FIRE, BROTHERHOOD, RIDDLE_OF_WIND = 69, 7395, 7396, 25766
local THUNDERCLAP = 25762
local MASTERFUL_BLITZ, FIRES_REPLY, WINDS_REPLY = 25764, 36950, 36949

local S_OPO_OPO, S_RAPTOR, S_COEURL, S_PERFECT_BALANCE, S_FORMLESS = 107, 108, 109, 110, 2513
local S_FIRES_RUMINATION, S_WINDS_RUMINATION, S_RIDDLE_OF_FIRE = 3843, 3842, 1181

local function opo(b)
    if b:aoe(3) then
        local area = b:best(SHADOW_OF_THE_DESTROYER, ARM_OF_THE_DESTROYER)
        if area then return area end
    end
    if b:has(DRAGON_KICK) and b:gauge_of("opo_fury") == 0 then return DRAGON_KICK end
    return b:best(LEAPING_OPO, BOOTSHINE)
end

local function raptor(b)
    if b:aoe(3) and b:has(FOUR_POINT_FURY) then return FOUR_POINT_FURY end
    if b:has(TWIN_SNAKES) and b:gauge_of("raptor_fury") == 0 then return TWIN_SNAKES end
    return b:best(RISING_RAPTOR, TRUE_STRIKE)
end

local function coeurl(b)
    if b:aoe(3) and b:has(ROCKBREAKER) then return ROCKBREAKER end
    if b:has(DEMOLISH) and b:gauge_of("coeurl_fury") == 0 then return DEMOLISH end
    return b:best(POUNCING_COEURL, SNAP_PUNCH)
end

-- Which form its last GCD action left it in (each form's actions lead to the next: Opo-opo to Raptor, Raptor to
-- Coeurl, Coeurl back to Opo-opo). Read from its last action rather than its form statuses, which the server leaves
-- on side by side.
local LEADS_TO_RAPTOR = { [BOOTSHINE] = true, [DRAGON_KICK] = true, [LEAPING_OPO] = true, [ARM_OF_THE_DESTROYER] = true, [SHADOW_OF_THE_DESTROYER] = true }
local LEADS_TO_COEURL = { [TRUE_STRIKE] = true, [TWIN_SNAKES] = true, [RISING_RAPTOR] = true, [FOUR_POINT_FURY] = true }

-- The Beast Chakra open (two bits each on the gauge.)
local function chakra_slots(b)
    local chakra, out = b:gauge_of("beast_chakra"), {}
    for slot = 0, 2 do
        local kind = (chakra >> (slot * 2)) & 3
        if kind ~= 0 then out[#out + 1] = kind end
    end
    return out
end

local function beast_chakra(b) return #chakra_slots(b) end

-- Under Perfect Balance: three Opo-opo actions for the Lunar Nadi, else one of each form for the Solar (from 60; any
-- form's action before it.)
local function balanced(b)
    if not b:has(MASTERFUL_BLITZ) then return opo(b) end
    local nadi = b:gauge_of("nadi")
    if nadi & 1 == 0 or nadi == 3 then return opo(b) end
    local seen = {}
    for _, kind in ipairs(chakra_slots(b)) do seen[kind] = true end
    if not seen[3] and b:has(SNAP_PUNCH) and b:gauge_of("coeurl_fury") == 0 then return coeurl(b) end
    if not seen[2] and b:has(TRUE_STRIKE) then return raptor(b) end
    if not seen[3] and b:has(SNAP_PUNCH) then return coeurl(b) end
    return opo(b)
end

local function gcd(b)
    if not b:in_melee() then
        -- Out of reach: open a chakra while getting there.
        local meditation = b:best(FORBIDDEN_MEDITATION, STEELED_MEDITATION)
        if meditation and b:gauge_of("chakra") < 5 then return meditation, "self" end
        return nil
    end
    -- Three Beast Chakra open: the Masterful Blitz they make.
    if b:has(MASTERFUL_BLITZ) and beast_chakra(b) >= 3 then return MASTERFUL_BLITZ end
    if b:has_status(S_PERFECT_BALANCE) then return balanced(b) end
    if b:has_status(S_FIRES_RUMINATION) and b:has(FIRES_REPLY) and not LEADS_TO_RAPTOR[b.combo] then return FIRES_REPLY end
    if b:has_status(S_WINDS_RUMINATION) and b:has(WINDS_REPLY) then return WINDS_REPLY end
    if b:has_status(S_FORMLESS) then return opo(b) end
    if LEADS_TO_RAPTOR[b.combo] and b:has(TRUE_STRIKE) then return raptor(b) end
    if LEADS_TO_COEURL[b.combo] and b:has(SNAP_PUNCH) then return coeurl(b) end
    return opo(b)
end

local function ogcd(b)
    if not b:in_melee() then
        if b:ready(THUNDERCLAP) and b.distance <= 20 then return THUNDERCLAP end
        return nil
    end
    if b:ready(RIDDLE_OF_FIRE) then return RIDDLE_OF_FIRE, "self" end
    if b:ready(BROTHERHOOD) then return BROTHERHOOD, "self" end
    if b:ready(RIDDLE_OF_WIND) then return RIDDLE_OF_WIND, "self" end
    -- Perfect Balance from Opo-opo form (after a Coeurl action), not over a Fury still to be spent.
    if b:ready(PERFECT_BALANCE) and not b:has_status(S_PERFECT_BALANCE)
        and not LEADS_TO_RAPTOR[b.combo] and not LEADS_TO_COEURL[b.combo] then
        return PERFECT_BALANCE, "self"
    end
    if b:gauge_of("chakra") >= 5 then
        local spend = b:aoe(3) and b:best(ENLIGHTENMENT, HOWLING_FIST) or b:best(FORBIDDEN_CHAKRA, STEEL_PEAK)
        if spend and b:ready(spend) then return spend end
    end
    return nil
end

return {
    classjobs = { 2, 20 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
