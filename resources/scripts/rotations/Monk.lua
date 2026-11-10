-- Pugilist and Monk: their rotation, 1-100.
-- Priorities from Boss Mod's standard Monk rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Monk guide (7.x):
-- the three forms in turn (Opo-opo, Raptor, Coeurl); in each, the action that builds its Fury when it has none
-- (Dragon Kick, Twin Snakes, Demolish) and otherwise the one that spends it (Bootshine / Leaping Opo, True Strike /
-- Rising Raptor, Snap Punch / Pouncing Coeurl), the area ones (Arm of the Destroyer, Four-point Fury, Rockbreaker) on
-- three or more; Perfect Balance for three Opo-opo actions; Steel Peak / The Forbidden Chakra at five chakra; Riddle
-- of Fire, Brotherhood and Riddle of Wind on cooldown. Retail's Monks at 50-59 (two players, 556 s) used Dragon Kick
-- 5.6 a minute, Bootshine 4.9, Snap Punch 3.2, True Strike 2.9, Twin Snakes 2.8, Demolish 1.6: Demolish's Fury lasts
-- two Snap Punches, the others' one (b.mem keeps the Fury it has built; the server has no Fury counters yet).

local BOOTSHINE, TRUE_STRIKE, SNAP_PUNCH = 53, 54, 56
local DRAGON_KICK, TWIN_SNAKES, DEMOLISH = 74, 61, 66
local LEAPING_OPO, RISING_RAPTOR, POUNCING_COEURL = 36945, 36946, 36947
local ARM_OF_THE_DESTROYER, SHADOW_OF_THE_DESTROYER, FOUR_POINT_FURY, ROCKBREAKER = 62, 25767, 16473, 70
local STEEL_PEAK, FORBIDDEN_CHAKRA, HOWLING_FIST, ENLIGHTENMENT = 25761, 3547, 25763, 16474
local STEELED_MEDITATION, FORBIDDEN_MEDITATION = 36940, 36942
local PERFECT_BALANCE, RIDDLE_OF_FIRE, BROTHERHOOD, RIDDLE_OF_WIND = 69, 7395, 7396, 25766
local THUNDERCLAP = 25762

local S_OPO_OPO, S_RAPTOR, S_COEURL, S_PERFECT_BALANCE, S_FORMLESS = 107, 108, 109, 110, 2513

local function opo(b)
    if b:aoe(3) then
        local area = b:best(SHADOW_OF_THE_DESTROYER, ARM_OF_THE_DESTROYER)
        if area then return area end
    end
    if b:has(DRAGON_KICK) and (b.mem.opo or 0) == 0 then
        b.mem.opo = 1
        return DRAGON_KICK
    end
    b.mem.opo = 0
    return b:best(LEAPING_OPO, BOOTSHINE)
end

local function raptor(b)
    if b:aoe(3) and b:has(FOUR_POINT_FURY) then return FOUR_POINT_FURY end
    if b:has(TWIN_SNAKES) and (b.mem.raptor or 0) == 0 then
        b.mem.raptor = 1
        return TWIN_SNAKES
    end
    b.mem.raptor = 0
    return b:best(RISING_RAPTOR, TRUE_STRIKE)
end

local function coeurl(b)
    if b:aoe(3) and b:has(ROCKBREAKER) then return ROCKBREAKER end
    if b:has(DEMOLISH) and (b.mem.coeurl or 0) == 0 then
        b.mem.coeurl = 2
        return DEMOLISH
    end
    b.mem.coeurl = math.max((b.mem.coeurl or 0) - 1, 0)
    return b:best(POUNCING_COEURL, SNAP_PUNCH)
end

-- Which form its last GCD action left it in (each form's actions lead to the next: Opo-opo to Raptor, Raptor to
-- Coeurl, Coeurl back to Opo-opo). Read from its last action rather than its form statuses, which the server leaves
-- on side by side.
local LEADS_TO_RAPTOR = { [BOOTSHINE] = true, [DRAGON_KICK] = true, [LEAPING_OPO] = true, [ARM_OF_THE_DESTROYER] = true, [SHADOW_OF_THE_DESTROYER] = true }
local LEADS_TO_COEURL = { [TRUE_STRIKE] = true, [TWIN_SNAKES] = true, [RISING_RAPTOR] = true, [FOUR_POINT_FURY] = true }

local function gcd(b)
    if not b:in_melee() then
        -- Out of reach: open a chakra while getting there.
        local meditation = b:best(FORBIDDEN_MEDITATION, STEELED_MEDITATION)
        if meditation and b:gauge_of("chakra") < 5 then return meditation, "self" end
        return nil
    end
    if b:has_status(S_PERFECT_BALANCE) or b:has_status(S_FORMLESS) then return opo(b) end
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
