-- Lancer and Dragoon: their rotation, 1-100.
-- Priorities from Boss Mod's standard Dragoon rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Dragoon guide
-- (7.x): two combos from True Thrust, Disembowel's (Power Surge, then Chaos Thrust / Chaotic Spring and Wheeling
-- Thrust) when Power Surge or the damage over time is running low, otherwise Vorpal Thrust's (Full Thrust / Heavens'
-- Thrust, then Fang and Claw, Drakesbane); Doom Spike - Sonic Thrust - Coerthan Torment on three or more; Lance Charge
-- and Battle Litany on cooldown, Life Surge before the strongest finisher, the jumps and Geirskogul inside Power Surge.
-- Retail's Lancers at 20-29 (two players, 482 s) used True Thrust 8.3 a minute, Vorpal Thrust 6.1, Full Thrust 4.0,
-- Disembowel 2.4, Life Surge 1.1; its Dragoons at 30-39 True Thrust 7.5, Vorpal 5.1, Full Thrust 4.8, Jump 2.3,
-- Disembowel 1.7, Life Surge 1.3, Lance Charge 0.9.

local TRUE_THRUST, VORPAL_THRUST, FULL_THRUST, HEAVENS_THRUST = 75, 78, 84, 25771
local DISEMBOWEL, CHAOS_THRUST, CHAOTIC_SPRING = 87, 88, 25772
local RAIDEN_THRUST, LANCE_BARRAGE, SPIRAL_BLOW = 16479, 36954, 36955
local FANG_AND_CLAW, WHEELING_THRUST, DRAKESBANE = 3554, 3556, 36952
local DOOM_SPIKE, DRACONIAN_FURY, SONIC_THRUST, COERTHAN_TORMENT = 86, 25770, 7397, 16477
local PIERCING_TALON = 90
local LIFE_SURGE, LANCE_CHARGE, BATTLE_LITANY = 83, 85, 3557
local JUMP, HIGH_JUMP, MIRAGE_DIVE, DRAGONFIRE_DIVE, RISE_OF_THE_DRAGON = 92, 16478, 7399, 96, 36953
local GEIRSKOGUL, NASTROND, STARDIVER, STARCROSS, WYRMWIND_THRUST = 3555, 7400, 16480, 36956, 25773

local S_POWER_SURGE, S_LIFE_SURGE, S_LANCE_CHARGE = 2720, 116, 1864
local S_CHAOS_THRUST, S_CHAOTIC_SPRING = 118, 2719
local S_DIVE_READY, S_NASTROND_READY, S_DRAGONS_FLIGHT, S_STARCROSS_READY = 1243, 3844, 3845, 3846
local S_DRACONIAN_FIRE = 1863

local function dot_left(b)
    return math.max(b:dot(S_CHAOS_THRUST), b:dot(S_CHAOTIC_SPRING))
end

local function gcd(b)
    if not b:in_melee() and b:has(PIERCING_TALON) then return PIERCING_TALON end
    -- Finish a combo under way.
    if b.combo == WHEELING_THRUST or b.combo == FANG_AND_CLAW then
        if b:has(DRAKESBANE) then return DRAKESBANE end
    end
    if b.combo == FULL_THRUST or b.combo == HEAVENS_THRUST then
        if b:has(FANG_AND_CLAW) then return FANG_AND_CLAW end
    end
    if b.combo == CHAOS_THRUST or b.combo == CHAOTIC_SPRING then
        if b:has(WHEELING_THRUST) then return WHEELING_THRUST end
    end
    if b.combo == VORPAL_THRUST or b.combo == LANCE_BARRAGE then
        local finisher = b:best(HEAVENS_THRUST, FULL_THRUST)
        if finisher then return finisher end
    end
    if b.combo == DISEMBOWEL or b.combo == SPIRAL_BLOW then
        local finisher = b:best(CHAOTIC_SPRING, CHAOS_THRUST)
        if finisher then return finisher end
    end
    if b.combo == SONIC_THRUST and b:has(COERTHAN_TORMENT) then return COERTHAN_TORMENT end
    if (b.combo == DOOM_SPIKE or b.combo == DRACONIAN_FURY) and b:has(SONIC_THRUST) then return SONIC_THRUST end
    if (b.combo == TRUE_THRUST or b.combo == RAIDEN_THRUST) then
        -- Disembowel's combo while Power Surge or the damage over time runs low, else Vorpal Thrust's.
        local surge = b:best(SPIRAL_BLOW, DISEMBOWEL)
        if surge and (b:status(S_POWER_SURGE) < 10 or (b:has(CHAOS_THRUST) and dot_left(b) < 10)) then return surge end
        local vorpal = b:best(LANCE_BARRAGE, VORPAL_THRUST)
        if vorpal then return vorpal end
    end
    if b:aoe(3) and b:has(DOOM_SPIKE) then return b:best(DRACONIAN_FURY, DOOM_SPIKE) end
    if b:has_status(S_DRACONIAN_FIRE) and b:has(RAIDEN_THRUST) then return RAIDEN_THRUST end
    return TRUE_THRUST
end

local function ogcd(b)
    if not b:in_melee() then return nil end
    local surged = b:has_status(S_POWER_SURGE) or not b:has(DISEMBOWEL)
    if b:ready(LANCE_CHARGE) and surged then return LANCE_CHARGE, "self" end
    if b:ready(BATTLE_LITANY) and surged then return BATTLE_LITANY, "self" end
    -- Life Surge before the strongest finisher coming next.
    if b:ready(LIFE_SURGE) and not b:has_status(S_LIFE_SURGE)
        and (b.combo == VORPAL_THRUST or b.combo == LANCE_BARRAGE or b.combo == WHEELING_THRUST
            or b.combo == FANG_AND_CLAW or (not b:has(VORPAL_THRUST))) then
        return LIFE_SURGE, "self"
    end
    if not surged then return nil end
    if b:has_status(S_NASTROND_READY) and b:ready(NASTROND) then return NASTROND end
    if b:ready(GEIRSKOGUL) then return GEIRSKOGUL end
    local jump = b:best(HIGH_JUMP, JUMP)
    if jump and b:ready(jump) then return jump end
    if b:has_status(S_DIVE_READY) and b:ready(MIRAGE_DIVE) then return MIRAGE_DIVE end
    if b:ready(DRAGONFIRE_DIVE) then return DRAGONFIRE_DIVE end
    if b:has_status(S_DRAGONS_FLIGHT) and b:ready(RISE_OF_THE_DRAGON) then return RISE_OF_THE_DRAGON end
    if b:ready(STARDIVER) and b:has_status(S_NASTROND_READY) then return STARDIVER end
    if b:has_status(S_STARCROSS_READY) and b:ready(STARCROSS) then return STARCROSS end
    if b:ready(WYRMWIND_THRUST) and b:gauge_of("firstminds_focus") >= 2 then return WYRMWIND_THRUST end
    return nil
end

return {
    classjobs = { 4, 22 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
