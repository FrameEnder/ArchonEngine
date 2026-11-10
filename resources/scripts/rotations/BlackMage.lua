-- Thaumaturge and Black Mage: their rotation, 1-100 as far as the server's Black Mage goes (Astral Fire, Umbral Ice and
-- Umbral Hearts are on the gauge, Global.lua's blm* functions; Enochian, Polyglot, Paradox and Astral Soul aren't yet,
-- so Fire IV, Despair, Xenoglossy and Flare Star are left out).
-- Priorities from Boss Mod's standard Black Mage rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Black Mage
-- leveling guide and the official job guide (7.x): in Astral Fire, Fire (Fire IV later) while the MP allows one more and
-- Blizzard III after it, Fire III at once on Firestarter; Blizzard III (Umbral Ice III gives all the MP back on landing)
-- then Fire III; before 35 Fire and Blizzard swapped by Transpose (Blizzard until the MP is full); Thunder whenever
-- Thunderhead is up and the target's runs low; on three or more Fire II / Flare and Blizzard II / Freeze. Retail's
-- Thaumaturge-Black Mages at 10-19 (two players, 795 s) cast Blizzard 8.2 a minute, Fire 4.0, Thunder 2.5, Transpose
-- 2.0; at 50-59 (347 s) Fire 10.9, Fire III 5.9, Blizzard III 1.9, Thunder III 4.0, Freeze 1.6, Flare 1.2.

local FIRE, FIRE_II, FIRE_III, FIRE_IV, FLARE = 141, 147, 152, 3577, 162
local BLIZZARD, BLIZZARD_II, BLIZZARD_III, FREEZE = 142, 25793, 154, 159
local THUNDER, THUNDER_II, THUNDER_III, THUNDER_IV, HIGH_THUNDER, HIGH_THUNDER_II = 144, 7447, 153, 7420, 36986, 36987
local TRANSPOSE, MANAFONT, UMBRAL_SOUL = 149, 158, 16506

local S_THUNDERHEAD, S_FIRESTARTER = 3870, 165
local S_THUNDER = { 161, 162, 163, 1210, 3871, 3872 }

local function thunder_left(b)
    local left = 0
    for _, s in ipairs(S_THUNDER) do left = math.max(left, b:dot(s)) end
    return left
end

-- A Fire spell's cost now (the gauge doubles it in Astral Fire, Umbral Hearts aside, and makes it free in Umbral Ice).
local function fire_cost(b, base)
    local element = b:gauge_of("element")
    if element < 0 then return 0 end
    if element > 0 and b:gauge_of("umbral_hearts") == 0 then return base * 2 end
    return base
end

local function gcd(b)
    local element = b:gauge_of("element")
    local mp, max = b.mp, b.max_mp
    if b:has_status(S_THUNDERHEAD) and thunder_left(b) < 3 and b.target_hp > 10 then
        local t = b:aoe(3) and b:best(HIGH_THUNDER_II, THUNDER_IV, THUNDER_II) or b:best(HIGH_THUNDER, THUNDER_III, THUNDER)
        if t then return t end
    end
    if b:aoe(3) and b:has(FIRE_II) then
        if element < 0 then
            if mp < max - 500 then
                local ice = b:best(FREEZE, BLIZZARD_II)
                if ice then return ice end
            end
            return b:best(FIRE_III, FIRE_II)
        end
        if mp >= fire_cost(b, 1500) + 800 then return FIRE_II end
        if b:has(FLARE) and element > 0 and mp >= 800 then return FLARE end
        return b:best(BLIZZARD_III, BLIZZARD_II)
    end
    if element > 0 then
        if b:has_status(S_FIRESTARTER) and b:has(FIRE_III) then return FIRE_III end
        local fire = b:best(FIRE_IV, FIRE)
        -- Enough left after it for Blizzard III (800 in Astral Fire), once there's Blizzard III to go to.
        local reserve = b:has(BLIZZARD_III) and 800 or 0
        if mp >= fire_cost(b, 800) + reserve then return fire end
        if b:has(BLIZZARD_III) then return BLIZZARD_III end
        -- Before 35: Transpose (an ability) swaps to Umbral Ice; Blizzard if it isn't learned.
        if b:has(TRANSPOSE) then return nil end
        return BLIZZARD
    end
    if element < 0 then
        if mp >= max - 400 then
            if b:has(FIRE_III) then return FIRE_III end
            if b:has(TRANSPOSE) then return nil end
            return FIRE
        end
        if b:has(BLIZZARD_III) and element > -3 then return BLIZZARD_III end
        return BLIZZARD
    end
    -- Neither: Fire with the MP for it, else Ice to get it back.
    if mp >= max / 2 then return b:best(FIRE_III, FIRE) end
    return b:best(BLIZZARD_III, BLIZZARD)
end

local function ogcd(b)
    local element = b:gauge_of("element")
    if b:has(TRANSPOSE) and b:ready(TRANSPOSE) and not b:has(FIRE_III) then
        -- Before Fire III and Blizzard III: Transpose when the MP has run out in Fire, or is full in Ice.
        if element > 0 and b.mp < fire_cost(b, 800) then return TRANSPOSE, "self" end
        if element < 0 and b.mp >= b.max_mp - 400 then return TRANSPOSE, "self" end
    end
    if b:ready(MANAFONT) and element > 0 and b.mp < 1600 then return MANAFONT, "self" end
    return nil
end

return {
    classjobs = { 7, 25 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
