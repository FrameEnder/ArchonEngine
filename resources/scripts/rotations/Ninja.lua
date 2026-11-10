-- Rogue and Ninja: their rotation, 1-100 (as far as the server's Ninjutsu goes: the mudras then Ninjutsu, one spell).
-- Priorities from Boss Mod's standard Ninja rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Ninja guide (7.x):
-- Spinning Edge - Gust Slash - Aeolian Edge (Armor Crush to keep Kazematoi, from 54), Death Blossom - Hakke Mujinsatsu
-- on three or more; a Ninjutsu whenever a mudra charge is back (two mudras then Ninjutsu: Raiton, or Fuma Shuriken
-- before Chi is learned), Kassatsu before one; Mug / Dokumori, Trick Attack / Kunai's Bane and Assassinate / Dream
-- Within a Dream on cooldown. Retail's Ninjas at 30-39 (two players, 1095 s) used Spinning Edge 6.5 a minute, Gust
-- Slash 6.2, Aeolian Edge 6.0, Ten 3.6, Fuma Shuriken 3.5, Mug 0.4; at 50-59 Chi 5.1, Raiton 3.4, Ten 3.7, Jin 1.2,
-- Kassatsu 1.3, Trick Attack 1.2, Assassinate 1.2.

local SPINNING_EDGE, GUST_SLASH, AEOLIAN_EDGE, ARMOR_CRUSH = 2240, 2242, 2255, 3563
local DEATH_BLOSSOM, HAKKE_MUJINSATSU = 2254, 16488
local THROWING_DAGGER = 2247
local TEN, CHI, JIN, NINJUTSU = 2259, 2261, 2263, 2260
local MUG, DOKUMORI, TRICK_ATTACK, KUNAIS_BANE = 2248, 36957, 2258, 36958
local ASSASSINATE, DREAM_WITHIN_A_DREAM, KASSATSU, HIDE = 2246, 3566, 2264, 2245
local BHAVACAKRA, HELLFROG_MEDIUM = 7402, 7401

local S_MUDRA, S_KASSATSU, S_HIDDEN, S_SHADOW_WALKER = 496, 497, 614, 3848

local function gcd(b)
    -- A mudra begun: finish the Ninjutsu.
    if b:has_status(S_MUDRA) and b:has(NINJUTSU) then
        if (b.mem.mudras or 0) >= 2 or not b:has(CHI) then
            b.mem.mudras = 0
            return NINJUTSU
        end
        return nil
    end
    if not b:in_melee() then
        if b:has(THROWING_DAGGER) then return THROWING_DAGGER end
        return nil
    end
    if b:aoe(3) and b:has(DEATH_BLOSSOM) then
        if b.combo == DEATH_BLOSSOM and b:has(HAKKE_MUJINSATSU) then return HAKKE_MUJINSATSU end
        return DEATH_BLOSSOM
    end
    if b.combo == GUST_SLASH then
        -- Armor Crush builds the Kazematoi Aeolian Edge spends.
        if b:has(ARMOR_CRUSH) and b:gauge_of("kazematoi") < 2 then return ARMOR_CRUSH end
        if b:has(AEOLIAN_EDGE) then return AEOLIAN_EDGE end
    end
    if b.combo == SPINNING_EDGE and b:has(GUST_SLASH) then return GUST_SLASH end
    return SPINNING_EDGE
end

local function ogcd(b)
    -- Mudras: the first when a charge is back, the second straight after.
    if b:has_status(S_MUDRA) then
        if (b.mem.mudras or 0) < 2 and b:has(CHI) then
            b.mem.mudras = (b.mem.mudras or 1) + 1
            return CHI, "self"
        end
        return nil
    end
    if not b:in_melee() then return nil end
    if b:ready(KASSATSU) and b:ready(TEN) then return KASSATSU, "self" end
    if b:ready(TEN) and b:has(NINJUTSU) then
        b.mem.mudras = 1
        return TEN, "self"
    end
    local mug = b:best(DOKUMORI, MUG)
    if mug and b:ready(mug) then return mug end
    local trick = b:best(KUNAIS_BANE, TRICK_ATTACK)
    if trick and b:ready(trick) and (b:has_status(S_HIDDEN) or b:has_status(S_SHADOW_WALKER) or trick == TRICK_ATTACK) then
        return trick
    end
    local dream = b:best(DREAM_WITHIN_A_DREAM, ASSASSINATE)
    if dream and b:ready(dream) then return dream end
    if b:gauge_of("ninki") >= 50 then
        local spend = b:aoe(3) and b:best(HELLFROG_MEDIUM) or b:best(BHAVACAKRA)
        if spend and b:ready(spend) then return spend end
    end
    return nil
end

return {
    classjobs = { 29, 30 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
