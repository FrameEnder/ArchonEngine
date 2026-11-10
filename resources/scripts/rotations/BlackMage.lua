-- Thaumaturge and Black Mage: their rotation, 1-100 (the element, Umbral Hearts, Polyglot, Paradox and Astral Soul are
-- on the server's gauge, Global.lua's blm* functions).
-- Priorities from Boss Mod's standard Black Mage rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Black Mage guide
-- and the official job guide (7.x): in Astral Fire, Fire IV (Fire before 60) while the MP allows one more and Despair
-- (from 72) after it, Paradox once, Flare Star at six Astral Soul, Fire III at once on Firestarter, Despair with the
-- last of the MP, Manafont then for another round; Blizzard III (Umbral Ice III gives the MP back), Blizzard IV for the
-- Umbral Hearts (58), Paradox, then Fire III; before 35 Fire and Blizzard swapped by Transpose (Blizzard until the MP is
-- full); Xenoglossy (Foul before 80) before Polyglot would overflow, Amplifier and Ley Lines on cooldown; Thunder
-- whenever Thunderhead is up and the target's runs low; on three or more Fire II / High Fire II and Flare, Blizzard II
-- / High Blizzard II and Freeze, Foul. Retail's Thaumaturge-Black Mages at 10-19 (two players, 795 s) cast Blizzard 8.2
-- a minute, Fire 4.0, Thunder 2.5, Transpose 2.0; at 50-59 (347 s) Fire 10.9, Fire III 5.9, Blizzard III 1.9, Thunder
-- III 4.0; at 80-89 (five players) Fire IV 338 casts to Despair's 49, Xenoglossy 63, Fire III 55, Blizzard III 44,
-- Blizzard IV 43, Thunder III 77, Triplecast 28, Ley Lines 27, Manafont 12.

local FIRE, FIRE_II, FIRE_III, FIRE_IV, FLARE = 141, 147, 152, 3577, 162
local BLIZZARD, BLIZZARD_II, BLIZZARD_III, FREEZE = 142, 25793, 154, 159
local THUNDER, THUNDER_II, THUNDER_III, THUNDER_IV, HIGH_THUNDER, HIGH_THUNDER_II = 144, 7447, 153, 7420, 36986, 36987
local TRANSPOSE, MANAFONT, UMBRAL_SOUL = 149, 158, 16506
local BLIZZARD_IV, DESPAIR, PARADOX, XENOGLOSSY, FOUL, FLARE_STAR = 3576, 16505, 25797, 16507, 7422, 36989
local HIGH_FIRE_II, HIGH_BLIZZARD_II, LEY_LINES, AMPLIFIER = 25794, 25795, 3573, 25796

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

local function polyglot_max(b)
    if b.level >= 98 then return 3 end
    if b.level >= 80 then return 2 end
    return 1
end

local function gcd(b)
    local element = b:gauge_of("element")
    local mp, max = b.mp, b.max_mp
    if b:has_status(S_THUNDERHEAD) and thunder_left(b) < 3 and b.target_hp > 10 then
        local t = b:aoe(3) and b:best(HIGH_THUNDER_II, THUNDER_IV, THUNDER_II) or b:best(HIGH_THUNDER, THUNDER_III, THUNDER)
        if t then return t end
    end
    -- Polyglot before it would overflow (or the next is about to come.)
    local polyglot = b:gauge_of("polyglot")
    if polyglot > 0 and (polyglot >= polyglot_max(b) or b:ready(AMPLIFIER)) then
        local spend = (b:aoe(3) or not b:has(XENOGLOSSY)) and FOUL or XENOGLOSSY
        if b:has(spend) then return spend end
    end
    if b:has(FLARE_STAR) and b:gauge_of("astral_soul") >= 6 then return FLARE_STAR end
    if b:aoe(3) and b:has(FIRE_II) then
        if element < 0 then
            if b:gauge_of("umbral_hearts") < 3 and b:has(FREEZE) then return FREEZE end
            if mp < max - 500 then
                local ice = b:best(HIGH_BLIZZARD_II, BLIZZARD_II)
                if ice then return ice end
            end
            return b:best(HIGH_FIRE_II, FIRE_III, FIRE_II)
        end
        if element > 0 and b:has(FLARE) and mp >= 800 and (mp < fire_cost(b, 1500) + 800 or b:gauge_of("umbral_hearts") <= 1) then
            return FLARE
        end
        if mp >= fire_cost(b, 1500) + 800 then return b:best(HIGH_FIRE_II, FIRE_II) end
        return b:best(HIGH_BLIZZARD_II, BLIZZARD_III, BLIZZARD_II)
    end
    if element > 0 then
        if b:has_status(S_FIRESTARTER) and b:has(FIRE_III) and (element < 3 or not b:has(FIRE_IV)) then return FIRE_III end
        if b:has(PARADOX) and b:gauge_of("paradox") > 0 and mp >= 1600 + 800 then return PARADOX end
        local fire = b:best(FIRE_IV, FIRE)
        -- Enough left after it for Despair (800) or for Blizzard III (free in Astral Fire, but 800 before Fire IV.)
        local reserve = 0
        if b:has(DESPAIR) then reserve = 800 elseif b:has(BLIZZARD_III) and not b:has(FIRE_IV) then reserve = 800 end
        if mp >= fire_cost(b, 800) + reserve then return fire end
        if b:has(DESPAIR) and mp >= 800 then return DESPAIR end
        if b:has_status(S_FIRESTARTER) and b:has(FIRE_III) then return FIRE_III end
        -- Manafont's MP comes first if it's ready (an ability: see ogcd).
        if b:ready(MANAFONT) and b:has(DESPAIR) then return nil end
        if b:has(BLIZZARD_III) then return BLIZZARD_III end
        -- Before 35: Transpose (an ability) swaps to Umbral Ice; Blizzard if it isn't learned.
        if b:has(TRANSPOSE) then return nil end
        return BLIZZARD
    end
    if element < 0 then
        if b:has(BLIZZARD_IV) and b:gauge_of("umbral_hearts") < 3 then return BLIZZARD_IV end
        if b:has(PARADOX) and b:gauge_of("paradox") > 0 then return PARADOX end
        if mp >= max - 400 then
            if b:has(FIRE_III) then return FIRE_III end
            if b:has(TRANSPOSE) then return nil end
            -- (Level 1 has Blizzard alone.)
            if b:has(FIRE) then return FIRE end
        end
        if b:has(BLIZZARD_III) and element > -3 then return BLIZZARD_III end
        if polyglot > 0 and b:has(FOUL) then return b:has(XENOGLOSSY) and XENOGLOSSY or FOUL end
        return BLIZZARD
    end
    -- Neither: Fire with the MP for it, else Ice to get it back.
    if mp >= max / 2 and b:has(FIRE) then return b:best(FIRE_III, FIRE) end
    return b:best(BLIZZARD_III, BLIZZARD)
end

local function ogcd(b)
    local element = b:gauge_of("element")
    if b:has(TRANSPOSE) and b:ready(TRANSPOSE) and not b:has(FIRE_III) then
        -- Before Fire III and Blizzard III: Transpose when the MP has run out in Fire, or is full in Ice.
        if element > 0 and b.mp < fire_cost(b, 800) then return TRANSPOSE, "self" end
        if element < 0 and b.mp >= b.max_mp - 400 then return TRANSPOSE, "self" end
    end
    -- Manafont once the MP is spent in Astral Fire (after Despair from 72.)
    if b:ready(MANAFONT) and element > 0 and b.mp < 800 then return MANAFONT, "self" end
    if b:ready(AMPLIFIER) and b:gauge_of("polyglot") < polyglot_max(b) and element ~= 0 then return AMPLIFIER, "self" end
    if b:ready(LEY_LINES) and element > 0 then return LEY_LINES, "self" end
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
