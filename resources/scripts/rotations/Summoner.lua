-- Arcanist and Summoner: their rotation, 1-50 as far as the server's Summoner goes (its arcana and attunement aren't on
-- the server yet, so b.mem keeps them here: a summon only after Aethercharge, Gemshine only while attuned).
-- Priorities from Boss Mod's standard Summoner rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Summoner guide
-- (7.x): Aethercharge on cooldown gives the three arcana; each summon in turn (Ruby / Ifrit, Topaz / Titan, Emerald /
-- Garuda) attunes it (Ruby 2, Topaz 4, Emerald 4 as an Arcanist; Ifrit 2, Titan 4, Garuda 4) for Gemshine, or Precious
-- Brilliance on three or more; Ruin (II) otherwise, Outburst on three or more; Energy Drain on cooldown and Fester
-- (Painflare on three or more) to spend its Aetherflow. Retail's Summoner at 30-39 in the Duty Finder (358 s): Outburst
-- 5.5 a minute (trash), Ruin II 4.4, Topaz Ruin II 3.0, Emerald Outburst 2.7, Fester 2.0, Ruby Ruin II 1.7, Aethercharge
-- 1.2 and each summon 1.0-1.2.

local RUIN, RUIN_II, OUTBURST = 163, 172, 16511
local GEMSHINE, PRECIOUS_BRILLIANCE = 25883, 25884
local AETHERCHARGE = 25800
local SUMMON_RUBY, SUMMON_TOPAZ, SUMMON_EMERALD = 25802, 25803, 25804
local SUMMON_IFRIT, SUMMON_TITAN, SUMMON_GARUDA = 25805, 25806, 25807
local ENERGY_DRAIN, FESTER, PAINFLARE = 16508, 181, 3578

-- Each arcanum: its summons (the later first) and the attunement each gives.
local ARCANA = {
    { SUMMON_IFRIT, 2, SUMMON_RUBY, 2 },
    { SUMMON_TITAN, 4, SUMMON_TOPAZ, 4 },
    { SUMMON_GARUDA, 4, SUMMON_EMERALD, 4 },
}

local function gcd(b)
    local mem = b.mem
    if (mem.attuned or 0) > 0 then
        local gem = (b:aoe(3) and b:has(PRECIOUS_BRILLIANCE)) and PRECIOUS_BRILLIANCE or GEMSHINE
        if b:has(gem) then
            mem.attuned = mem.attuned - 1
            return gem
        end
        mem.attuned = 0
    end
    if mem.arcana then
        for i, arcanum in ipairs(ARCANA) do
            if mem.arcana[i] then
                local summon, stacks
                if b:has(arcanum[1]) then summon, stacks = arcanum[1], arcanum[2]
                elseif b:has(arcanum[3]) then summon, stacks = arcanum[3], arcanum[4] end
                mem.arcana[i] = nil
                if summon then
                    mem.attuned = stacks
                    return summon
                end
            end
        end
        mem.arcana = nil
    end
    if b:aoe(3) and b:has(OUTBURST) and b:affords(OUTBURST) then return OUTBURST end
    local ruin = b:best(RUIN_II, RUIN)
    if ruin and b:affords(ruin) then return ruin end
    return nil
end

local function ogcd(b)
    local mem = b.mem
    if b:ready(AETHERCHARGE) and not mem.arcana and (mem.attuned or 0) == 0 then
        mem.arcana = { true, true, true }
        return AETHERCHARGE, "self"
    end
    if b:ready(ENERGY_DRAIN) and b:gauge_of("aetherflow") == 0 then return ENERGY_DRAIN end
    if b:gauge_of("aetherflow") > 0 then
        local spend = (b:aoe(3) and b:has(PAINFLARE)) and PAINFLARE or FESTER
        if b:ready(spend) then return spend end
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
