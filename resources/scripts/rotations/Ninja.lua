-- Rogue and Ninja: their rotation, 1-100.
-- Priorities from Boss Mod's standard Ninja rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Ninja guide and the
-- official job guide (7.x): Spinning Edge - Gust Slash - Aeolian Edge (Armor Crush to keep Kazematoi, from 54), Death
-- Blossom - Hakke Mujinsatsu on three or more; a Ninjutsu whenever a mudra charge is back, its mudras chosen for it
-- (the gauge's "mudras", Global.lua's ninNinjutsuOf): Suiton (Ten Chi Jin) before Trick Attack / Kunai's Bane, which
-- need Shadow Walker, Katon (Chi Ten) on three or more, else Raiton (Ten Chi), Fuma Shuriken before Chi is learned;
-- under Kassatsu Hyosho Ranryu (Ten Jin) from 76; Ten Chi Jin's three Ninjutsu (Fuma Shuriken, Raiton, Suiton); Raiju
-- after Raiton, Phantom Kamaitachi after Bunshin; Mug / Dokumori, Trick Attack / Kunai's Bane and Assassinate / Dream
-- Within a Dream on cooldown, Meisui for Shadow Walker's Ninki when Trick Attack is far off, Ninki spent from 50
-- (Bunshin first, then Bhavacakra / Zesho Meppo, Hellfrog Medium on three or more). Retail's Ninjas at 30-39 (two
-- players, 1095 s) used Spinning Edge 6.5 a minute, Gust Slash 6.2, Aeolian Edge 6.0, Ten 3.6, Fuma Shuriken 3.5, Mug
-- 0.4; at 50-59 Ten 3.7 and its second mudra (the client sends 18805-18807 for it) 5.6, Raiton 3.4, Kassatsu 1.3,
-- Suiton 1.2 then Trick Attack 1.2, Assassinate 1.2.

local SPINNING_EDGE, GUST_SLASH, AEOLIAN_EDGE, ARMOR_CRUSH = 2240, 2242, 2255, 3563
local DEATH_BLOSSOM, HAKKE_MUJINSATSU = 2254, 16488
local THROWING_DAGGER = 2247
local TEN, CHI, JIN, NINJUTSU = 2259, 2261, 2263, 2260
local FUMA_SHURIKEN, RAITON, SUITON = 2265, 2267, 2271
local MUG, DOKUMORI, TRICK_ATTACK, KUNAIS_BANE = 2248, 36957, 2258, 36958
local ASSASSINATE, DREAM_WITHIN_A_DREAM, KASSATSU = 2246, 3566, 2264
local BHAVACAKRA, HELLFROG_MEDIUM, ZESHO_MEPPO, DEATHFROG_MEDIUM = 7402, 7401, 36960, 36959
local TEN_CHI_JIN, MEISUI, BUNSHIN, TENRI_JINDO = 7403, 16489, 16493, 36961
local PHANTOM_KAMAITACHI, FLEETING_RAIJU, FORKED_RAIJU = 25774, 25778, 25777

local S_MUDRA, S_KASSATSU, S_HIDDEN, S_SHADOW_WALKER = 496, 497, 614, 3848
local S_TEN_CHI_JIN, S_RAIJU_READY, S_PHANTOM_KAMAITACHI_READY = 1186, 2690, 2723
local S_HIGI, S_TENRI_JINDO_READY = 3850, 3851

-- The mudras (1 Ten, 2 Chi, 3 Jin) and the order that makes each Ninjutsu (the job guide.)
local MUDRA_ACTION = { TEN, CHI, JIN }
local PLANS = {
    fuma = { 1 },
    raiton = { 1, 2 },
    katon = { 2, 1 },
    hyosho = { 1, 3 },
    suiton = { 1, 2, 3 },
}

local function mudras_done(b)
    local mudras, count = b:gauge_of("mudras"), 0
    while count < 3 and (mudras >> (count * 2)) & 3 ~= 0 do count = count + 1 end
    return count
end

local function trick_soon(b)
    local trick = b:best(KUNAIS_BANE, TRICK_ATTACK)
    return trick and b:cd(trick) < 20
end

-- Which Ninjutsu to make now.
local function plan(b)
    if not b:has(CHI) then return PLANS.fuma end
    if b:has_status(S_KASSATSU) and b.level >= 76 and b:has(JIN) then return PLANS.hyosho end
    if b:has(JIN) and trick_soon(b) and not b:has_status(S_SHADOW_WALKER) and not b:has_status(S_KASSATSU) then
        return PLANS.suiton
    end
    if b:aoe(3) then return PLANS.katon end
    return PLANS.raiton
end

local function gcd(b)
    -- Ten Chi Jin: its three Ninjutsu in turn.
    if b:has_status(S_TEN_CHI_JIN) then
        local step = (b.mem.tcj or 0) + 1
        b.mem.tcj = step
        if step == 1 then return FUMA_SHURIKEN end
        if step == 2 then return RAITON end
        b.mem.tcj = nil
        return SUITON
    end
    b.mem.tcj = nil
    -- A mudra begun: the next one of the plan, or the Ninjutsu they make (mudras are abilities; see ogcd).
    if b:has_status(S_MUDRA) and b:has(NINJUTSU) then
        local p = b.mem.plan or PLANS.fuma
        if mudras_done(b) >= #p then
            b.mem.plan = nil
            return NINJUTSU
        end
        return nil
    end
    if not b:in_melee() then
        if b:has(THROWING_DAGGER) then return THROWING_DAGGER end
        return nil
    end
    if b:has_status(S_PHANTOM_KAMAITACHI_READY) and b:has(PHANTOM_KAMAITACHI) then return PHANTOM_KAMAITACHI end
    if b:has_status(S_RAIJU_READY) and b:has(FLEETING_RAIJU) then
        return b:aoe(3) and FORKED_RAIJU or FLEETING_RAIJU
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
    if b:has_status(S_TEN_CHI_JIN) then return nil end
    -- Mudras: the next of the plan.
    if b:has_status(S_MUDRA) then
        local p = b.mem.plan or PLANS.fuma
        local done = mudras_done(b)
        if done < #p and b:has(MUDRA_ACTION[p[done + 1]]) then
            return MUDRA_ACTION[p[done + 1]], "self"
        end
        return nil
    end
    if not b:in_melee() then return nil end
    if b:ready(KASSATSU) and b:ready(TEN) and b:has(NINJUTSU) then return KASSATSU, "self" end
    if b:has(NINJUTSU) and (b:ready(TEN) or b:has_status(S_KASSATSU)) then
        local p = plan(b)
        if b:has(MUDRA_ACTION[p[1]]) then
            b.mem.plan = p
            return MUDRA_ACTION[p[1]], "self"
        end
    end
    local trick = b:best(KUNAIS_BANE, TRICK_ATTACK)
    local hidden = b:has_status(S_HIDDEN) or b:has_status(S_SHADOW_WALKER)
    if trick and b:ready(trick) and (hidden or not b:has(SUITON) and trick == TRICK_ATTACK) then return trick end
    local mug = b:best(DOKUMORI, MUG)
    if mug and b:ready(mug) then return mug end
    local dream = b:best(DREAM_WITHIN_A_DREAM, ASSASSINATE)
    if dream and b:ready(dream) then return dream end
    if b:has_status(S_TENRI_JINDO_READY) and b:ready(TENRI_JINDO) then return TENRI_JINDO end
    -- Ten Chi Jin with no mudra charge to spend and Trick Attack not about to need one.
    if b:ready(TEN_CHI_JIN) and not b:ready(TEN) and not b:has_status(S_KASSATSU) and not trick_soon(b) then
        return TEN_CHI_JIN, "self"
    end
    -- Meisui turns Shadow Walker into Ninki when Trick Attack won't want it.
    if b:ready(MEISUI) and b:has_status(S_SHADOW_WALKER) and trick and b:cd(trick) > 20 and b:gauge_of("ninki") <= 50 then
        return MEISUI, "self"
    end
    local ninki = b:gauge_of("ninki")
    if ninki >= 50 then
        if b:ready(BUNSHIN) then return BUNSHIN, "self" end
        if b:has_status(S_HIGI) then
            local spend = b:aoe(3) and b:best(DEATHFROG_MEDIUM) or b:best(ZESHO_MEPPO)
            if spend and b:ready(spend) then return spend end
        end
        -- Kept for Bunshin when it's nearly back.
        if not b:has(BUNSHIN) or b:cd(BUNSHIN) > 10 or ninki >= 90 then
            local spend = b:aoe(3) and b:best(HELLFROG_MEDIUM) or b:best(BHAVACAKRA)
            if spend and b:ready(spend) then return spend end
        end
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
