-- Arcanist and Summoner: their rotation, 1-100 (the arcana and attunement are on the server's gauge: "arcana" 1 Ruby,
-- 2 Topaz, 4 Emerald; "attunement_type" 1 fire, 2 earth, 3 wind and "attunement" its stacks; Global.lua's smn*).
-- Priorities from Boss Mod's standard Summoner rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Summoner guide and
-- the official job guide (7.x): the demi-summon on cooldown from 58 (Dreadwyrm Trance, Summon Bahamut from 70, Bahamut
-- and Phoenix in turn from 80, Solar Bahamut every other one from 100), its trance's Impulse (Astral, Fountain of Fire,
-- Umbral) or Flare on three or more, Enkindle and Astral Flow (Deathflare, Rekindle, Sunflare) inside it; then the
-- three arcana it gives (Aethercharge's before 58), Titan, Garuda and Ifrit, each attuning it for its gem spells
-- (Ruby, Topaz, Emerald Ruin / Rite, Outburst / Disaster / Catastrophe on three or more: the IDs the client sends), and
-- their Favors (Mountain Buster, Slipstream, Crimson Cyclone - Crimson Strike from 86); Ruin III / II, Ruin IV on
-- Further Ruin, Outburst / Tri-disaster on three or more otherwise; Searing Light on cooldown with Searing Flash after it,
-- Energy Drain / Energy Siphon for Aetherflow, spent on Fester / Necrotize (Painflare on three or more). Retail's
-- Summoner at 30-39 in the Duty Finder (358 s): Outburst 5.5 a minute (trash), Ruin II 4.4, Topaz Ruin II 3.0, Emerald
-- Outburst 2.7, Fester 2.0, Ruby Ruin II 1.7, Aethercharge 1.2 and each summon 1.0-1.2; at 80-89 (two players, 764 s)
-- Topaz Rite 54 uses, Emerald Rite 47, Astral Impulse 36, Fester 31, Ruby Rite 28, Fountain of Fire 27, Ruin III 23,
-- Energy Drain 16, each egi 12-14, Ruin IV 13, Searing Light 9, Summon Bahamut 8, Summon Phoenix 7.

local RUIN, RUIN_II, RUIN_III, RUIN_IV, OUTBURST, TRI_DISASTER = 163, 172, 3579, 7426, 16511, 25826
local AETHERCHARGE, DREADWYRM_TRANCE = 25800, 3581
local SUMMON_BAHAMUT, SUMMON_PHOENIX, SUMMON_SOLAR_BAHAMUT = 7427, 25831, 36992
local ENERGY_DRAIN, ENERGY_SIPHON, FESTER, PAINFLARE, NECROTIZE = 16508, 16510, 181, 3578, 36990
local SEARING_LIGHT, SEARING_FLASH = 25801, 36991
local MOUNTAIN_BUSTER, SLIPSTREAM, CRIMSON_CYCLONE, CRIMSON_STRIKE = 25836, 25837, 25835, 25885

local S_FURTHER_RUIN, S_TITANS_FAVOR, S_IFRITS_FAVOR, S_GARUDAS_FAVOR = 2701, 2853, 2724, 2725
local S_CRIMSON_STRIKE_READY, S_RUBYS_GLIMMER = 4403, 3873
local S_DREADWYRM_TRANCE, S_FIREBIRD_TRANCE = 3228, 3229

-- Each demi-summon: its summon, its trance's single and area spells, its Astral Flow and its Enkindle.
local DEMIS = {
    trance = { summon = DREADWYRM_TRANCE, single = 25820, area = 25821, flow = 3582 },
    bahamut = { summon = SUMMON_BAHAMUT, single = 25820, area = 25821, flow = 3582, enkindle = 7429 },
    phoenix = { summon = SUMMON_PHOENIX, single = 16514, area = 16515, flow = 25830, enkindle = 16516, flow_self = true },
    solar = { summon = SUMMON_SOLAR_BAHAMUT, single = 36994, area = 36995, flow = 36996, enkindle = 36998 },
}

-- The arcana in the order they're summoned (Titan, Garuda, Ifrit): its bit, and its summons from the latest.
local ARCANA = {
    { bit = 2, summons = { 25839, 25806, 25803 } },
    { bit = 4, summons = { 25840, 25807, 25804 } },
    { bit = 1, summons = { 25838, 25805, 25802 } },
}

-- The gem spells by element (1 fire, 2 earth, 3 wind), single target then area, from the latest.
local GEMS = {
    [1] = { { 25823, 25817, 25811, 25808 }, { 25832, 25827, 25814 } },
    [2] = { { 25824, 25818, 25812, 25809 }, { 25833, 25828, 25815 } },
    [3] = { { 25825, 25819, 25813, 25810 }, { 25834, 25829, 25816 } },
}

local function best_of(b, list)
    for _, action in ipairs(list) do
        if b:has(action) then return action end
    end
    return nil
end

-- The demi-summon due next (b.mem.demis counts those summoned.)
local function next_demi(b)
    local n = b.mem.demis or 0
    if b:has(SUMMON_SOLAR_BAHAMUT) then
        local cycle = { "solar", "bahamut", "solar", "phoenix" }
        return DEMIS[cycle[n % 4 + 1]]
    end
    if b:has(SUMMON_PHOENIX) then return n % 2 == 0 and DEMIS.bahamut or DEMIS.phoenix end
    if b:has(SUMMON_BAHAMUT) then return DEMIS.bahamut end
    if b:has(DREADWYRM_TRANCE) then return DEMIS.trance end
    return nil
end

local function in_trance(b)
    return b:has_status(S_DREADWYRM_TRANCE) or b:has_status(S_FIREBIRD_TRANCE)
end

local function gcd(b)
    local demi = b.mem.demi and DEMIS[b.mem.demi]
    if in_trance(b) and demi then
        local spell = b:aoe(3) and demi.area or demi.single
        if b:has(spell) then return spell end
    end
    if b:has_status(S_CRIMSON_STRIKE_READY) and b:has(CRIMSON_STRIKE) then return CRIMSON_STRIKE end
    if b:has_status(S_IFRITS_FAVOR) and b:has(CRIMSON_CYCLONE) then return CRIMSON_CYCLONE end
    if b:has_status(S_GARUDAS_FAVOR) and b:has(SLIPSTREAM) then return SLIPSTREAM end
    -- Attuned: the element's gem spell.
    local kind = b:gauge_of("attunement_type")
    if kind > 0 and b:gauge_of("attunement") > 0 and GEMS[kind] then
        local gem = b:aoe(3) and best_of(b, GEMS[kind][2]) or best_of(b, GEMS[kind][1])
        if gem then return gem end
    end
    -- An arcanum left: its summon.
    local arcana = b:gauge_of("arcana")
    if arcana > 0 and not in_trance(b) then
        for _, arcanum in ipairs(ARCANA) do
            if arcana & arcanum.bit ~= 0 then
                local summon = best_of(b, arcanum.summons)
                if summon then return summon end
            end
        end
    end
    if b:has_status(S_FURTHER_RUIN) and b:has(RUIN_IV) then return RUIN_IV end
    if b:aoe(3) then
        local area = b:best(TRI_DISASTER, OUTBURST)
        if area and b:affords(area) then return area end
    end
    local ruin = b:best(RUIN_III, RUIN_II, RUIN)
    if ruin and b:affords(ruin) then return ruin end
    return nil
end

local function ogcd(b)
    local mem = b.mem
    local arcana, attuned = b:gauge_of("arcana"), b:gauge_of("attunement") > 0
    -- The demi-summon (its recast is Aethercharge's: b.mem.demi_at keeps it from being tried twice in a minute.)
    local demi = next_demi(b)
    local since = b.in_combat - (mem.demi_at or -999)
    if demi and b:ready(demi.summon) and arcana == 0 and not attuned and (since >= 59 or since < 0) then
        mem.demis = (mem.demis or 0) + 1
        mem.demi_at = b.in_combat
        for name, d in pairs(DEMIS) do
            if d == demi then mem.demi = name end
        end
        return demi.summon, "self"
    end
    if not demi and b:ready(AETHERCHARGE) and arcana == 0 and not attuned then return AETHERCHARGE, "self" end
    if b:ready(SEARING_LIGHT) then return SEARING_LIGHT, "self" end
    if b:has_status(S_RUBYS_GLIMMER) and b:ready(SEARING_FLASH) then return SEARING_FLASH end
    local current = mem.demi and DEMIS[mem.demi]
    if in_trance(b) and current then
        if current.enkindle and b:ready(current.enkindle) then return current.enkindle end
        if b:ready(current.flow) then return current.flow, current.flow_self and "self" or nil end
    end
    if b:has_status(S_TITANS_FAVOR) and b:ready(MOUNTAIN_BUSTER) then return MOUNTAIN_BUSTER end
    local drain = (b:aoe(3) and b:has(ENERGY_SIPHON)) and ENERGY_SIPHON or ENERGY_DRAIN
    if b:ready(drain) and b:gauge_of("aetherflow") == 0 then return drain end
    if b:gauge_of("aetherflow") > 0 then
        local spend = (b:aoe(3) and b:has(PAINFLARE)) and PAINFLARE or b:best(NECROTIZE, FESTER)
        if spend and b:ready(spend) then return spend end
    end
    return nil
end

return {
    classjobs = { 26, 27 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
