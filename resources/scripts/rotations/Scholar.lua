-- Scholar (Arcanist's healing job; Arcanist itself is Summoner.lua): its damage, 30-100 (its healing is the party
-- bots' own, `party_bot.rs`).
-- Priorities from Boss Mod's standard Scholar rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Scholar guide
-- (7.x): Bio / Bio II / Biolysis kept on the target, Ruin / Broil otherwise, Art of War on three or more around it,
-- Ruin II while it has to move; Aetherflow when it has none and Energy Drain to spend it (the Faerie Gauge fills from
-- 70), Chain Stratagem on cooldown with Baneful Impaction after it (92), Dissipation for three more Aetherflow.
-- Retail's Scholars at 50-59 (two players, 1035 s) cast Ruin 13.3 a minute, Bio II 3.8, Aetherflow 0.6, Energy Drain
-- 0.5; at 80-89 (five players, 1726 s) Broil III 365, Biolysis 66, Energy Drain 28, Aetherflow 28, Chain Stratagem
-- 15.

local RUIN, BROIL, BROIL_II, BROIL_III, BROIL_IV, RUIN_II = 17869, 3584, 7435, 16541, 25865, 17870
local BIO, BIO_II, BIOLYSIS = 17864, 17865, 16540
local ART_OF_WAR, ART_OF_WAR_II = 16539, 25866
local AETHERFLOW, ENERGY_DRAIN, CHAIN_STRATAGEM, BANEFUL_IMPACTION, DISSIPATION = 166, 167, 7436, 37012, 3587

local S_IMPACT_IMMINENT = 3882

local S_BIO, S_BIO_II, S_BIOLYSIS = 179, 189, 1895

local function gcd(b)
    local dot = b:best(BIOLYSIS, BIO_II, BIO)
    local left = math.max(b:dot(S_BIO), b:dot(S_BIO_II), b:dot(S_BIOLYSIS))
    if b.near_self >= 3 then
        local war = b:best(ART_OF_WAR_II, ART_OF_WAR)
        if war and b:affords(war) then return war end
    end
    if dot and left < 3 and b.target_hp > 10 and b:affords(dot) then return dot end
    if b.distance > 25 then return nil end
    local ruin = b:best(BROIL_IV, BROIL_III, BROIL_II, BROIL, RUIN)
    if ruin and b:affords(ruin) then return ruin end
    return nil
end

local function ogcd(b)
    if b:ready(CHAIN_STRATAGEM) then return CHAIN_STRATAGEM end
    if b:has_status(S_IMPACT_IMMINENT) and b:ready(BANEFUL_IMPACTION) then return BANEFUL_IMPACTION end
    if b:ready(DISSIPATION) and b:gauge_of("aetherflow") == 0 and not b:ready(AETHERFLOW) then return DISSIPATION, "self" end
    if b:ready(AETHERFLOW) and b:gauge_of("aetherflow") == 0 then return AETHERFLOW, "self" end
    if b:gauge_of("aetherflow") > 0 and b:ready(ENERGY_DRAIN) then return ENERGY_DRAIN end
    return nil
end

return {
    classjobs = { 28 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
