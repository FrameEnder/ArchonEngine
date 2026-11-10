-- Dark Knight: its rotation, 1-100.
-- Priorities from Boss Mod's standard Dark Knight rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Dark Knight guide
-- and the official job guide (7.x): Hard Slash - Syphon Strike - Souleater (Unleash - Stalwart Soul on three or
-- more), Bloodspiller / Quietus with the Blood (and under Delirium, its Scarlet Delirium - Comeuppance - Torcleaver chain
-- from 96), Disesteem on Scorn; Darkside kept up with Edge (Flood) of Darkness / Shadow and the MP spent before it fills,
-- The Blackest Night when hurt; Blood Weapon / Delirium, Living Shadow, Salted Earth and Salt and Darkness, Carve and
-- Spit / Abyssal Drain and Shadowbringer on cooldown; Shadow Wall / Shadowed Vigil and Dark Mind when the HP runs low.
-- Retail's Dark Knights at 50-59 (three players, 1667 s) used Hard Slash 178, Syphon Strike 163, Souleater 153, Edge of
-- Darkness 103, Unmend 55, Blood Weapon 24; at 80-89 (six players, 3036 s) Hard Slash 216, Bloodspiller 215, Syphon
-- Strike 211, Souleater 193, Edge of Shadow 135, The Blackest Night 49, Quietus 47, Delirium 43, Carve and Spit 43, Salted
-- Earth 40, Living Shadow 22.

local HARD_SLASH, SYPHON_STRIKE, SOULEATER, UNLEASH, STALWART_SOUL, UNMEND = 3617, 3623, 3632, 3621, 16468, 3624
local BLOODSPILLER, QUIETUS = 7392, 7391
local SCARLET_DELIRIUM, COMEUPPANCE, TORCLEAVER, IMPALEMENT, DISESTEEM = 36928, 36929, 36930, 36931, 36932
local EDGE_OF_DARKNESS, FLOOD_OF_DARKNESS, EDGE_OF_SHADOW, FLOOD_OF_SHADOW = 16467, 16466, 16470, 16469
local BLOOD_WEAPON, DELIRIUM, LIVING_SHADOW, SALTED_EARTH, SALT_AND_DARKNESS = 3625, 7390, 16472, 3639, 25755
local CARVE_AND_SPIT, ABYSSAL_DRAIN, SHADOWBRINGER, BLACKEST_NIGHT = 3643, 3641, 25757, 7393
local SHADOW_WALL, SHADOWED_VIGIL, DARK_MIND, LIVING_DEAD = 3636, 36927, 3634, 3638

local S_DELIRIUM, S_ENHANCED_DELIRIUM, S_SCORN, S_SALTED_EARTH, S_BLOOD_WEAPON = 1972, 3836, 3837, 749, 742

local function gcd(b)
    if not b:in_melee() then
        if b:has(UNMEND) then return UNMEND end
        return nil
    end
    -- Delirium's chain (96).
    if b:has_status(S_ENHANCED_DELIRIUM) then
        if b:aoe(3) then return IMPALEMENT end
        if b.combo == COMEUPPANCE then return TORCLEAVER end
        if b.combo == SCARLET_DELIRIUM then return COMEUPPANCE end
        return SCARLET_DELIRIUM
    end
    if b:has_status(S_SCORN) and b:has(DISESTEEM) then return DISESTEEM end
    local blood = b:gauge_of("blood")
    if b:has(BLOODSPILLER) and (b:has_status(S_DELIRIUM) or blood >= 50) then
        if b:aoe(3) and b:has(QUIETUS) then return QUIETUS end
        -- Spent before Souleater's 20 would overflow it, or while Blood Weapon's 10s come.
        if b:has_status(S_DELIRIUM) or blood >= 70 or b:has_status(S_BLOOD_WEAPON) or b.combo ~= SYPHON_STRIKE then
            return BLOODSPILLER
        end
    end
    if b:aoe(3) and b:has(UNLEASH) then
        if b.combo == UNLEASH and b:has(STALWART_SOUL) then return STALWART_SOUL end
        return UNLEASH
    end
    if b.combo == SYPHON_STRIKE and b:has(SOULEATER) then return SOULEATER end
    if b.combo == HARD_SLASH and b:has(SYPHON_STRIKE) then return SYPHON_STRIKE end
    return HARD_SLASH
end

local function ogcd(b)
    if not b:in_melee() then return nil end
    -- Mitigation when hurt.
    local wall = b:best(SHADOWED_VIGIL, SHADOW_WALL)
    if b.hp < 40 and b:ready(LIVING_DEAD) then return LIVING_DEAD, "self" end
    if b.hp < 55 and wall and b:ready(wall) then return wall, "self" end
    if b.hp < 75 and b:ready(DARK_MIND) then return DARK_MIND, "self" end
    if b.hp < 85 and b:ready(BLACKEST_NIGHT) and b.mp >= 3000 then return BLACKEST_NIGHT, "self" end
    local delirium = b:best(DELIRIUM, BLOOD_WEAPON)
    if delirium and b:ready(delirium) then return delirium, "self" end
    if b:ready(LIVING_SHADOW) then return LIVING_SHADOW, "self" end
    -- Darkside: kept up, and the MP (or Dark Arts) spent before it fills.
    local edge = b:aoe(3) and b:best(FLOOD_OF_SHADOW, FLOOD_OF_DARKNESS) or b:best(EDGE_OF_SHADOW, EDGE_OF_DARKNESS)
    if edge and b:ready(edge) and (b.mp >= 3000 or b:gauge_of("dark_arts") > 0) then
        if b:gauge_of("darkside") < 100 or b.mp >= 8000 or b:gauge_of("dark_arts") > 0 then return edge end
    end
    if b:ready(SALTED_EARTH) then return SALTED_EARTH, "self" end
    if b:ready(SALT_AND_DARKNESS) and b:dot(S_SALTED_EARTH) > 0 then return SALT_AND_DARKNESS, "self" end
    if b:ready(SHADOWBRINGER) and b:gauge_of("darkside") > 0 then return SHADOWBRINGER end
    local carve = (b:aoe(3) or not b:has(CARVE_AND_SPIT)) and ABYSSAL_DRAIN or CARVE_AND_SPIT
    if b:ready(carve) then return carve end
    return nil
end

return {
    classjobs = { 32 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
