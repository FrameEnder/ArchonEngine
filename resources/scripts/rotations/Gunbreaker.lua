-- Gunbreaker: its rotation, 1-100.
-- Priorities from Boss Mod's standard Gunbreaker rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Gunbreaker guide
-- and the official job guide (7.x): Keen Edge - Brutal Shell - Solid Barrel (Demon Slice - Demon Slaughter on three
-- or more) for the Cartridges; No Mercy on cooldown with Gnashing Fang - Savage Claw - Wicked Talon, Double Down, Sonic
-- Break and Bow Shock inside it, Bloodfest at no Cartridges and its Reign of Beasts - Noble Blood - Lion Heart; Burst
-- Strike (Fated Circle on three or more) inside No Mercy or before a Cartridge would be lost; each Continuation as it's
-- ready; Danger Zone / Blasting Zone on cooldown; Heart of Corundum (Stone), Camouflage, Nebula and Aurora when hurt.
-- Retail's Gunbreakers at 80-89 (19 players, 4114 s) used Keen Edge 279, Brutal Shell 268, Solid Barrel 253, Lightning
-- Shot 125, Gnashing Fang 123, Jugular Rip 119, Savage Claw 118, Abdomen Tear 114, Wicked Talon 112, Eye Gouge 109,
-- Blasting Zone 85, Burst Strike 75, Bow Shock 68, No Mercy 60, Sonic Break 60, Bloodfest 38.

local KEEN_EDGE, BRUTAL_SHELL, SOLID_BARREL, DEMON_SLICE, DEMON_SLAUGHTER = 16137, 16139, 16145, 16141, 16149
local LIGHTNING_SHOT, BURST_STRIKE, FATED_CIRCLE, SONIC_BREAK, DOUBLE_DOWN = 16143, 16162, 16163, 16153, 25760
local GNASHING_FANG, SAVAGE_CLAW, WICKED_TALON = 16146, 16147, 16150
local REIGN_OF_BEASTS, NOBLE_BLOOD, LION_HEART = 36937, 36938, 36939
local NO_MERCY, BLOODFEST, DANGER_ZONE, BLASTING_ZONE, BOW_SHOCK = 16138, 16164, 16144, 16165, 16159
local JUGULAR_RIP, ABDOMEN_TEAR, EYE_GOUGE, HYPERVELOCITY, FATED_BRAND = 16156, 16157, 16158, 25759, 36936
local HEART_OF_STONE, HEART_OF_CORUNDUM, CAMOUFLAGE, NEBULA, GREAT_NEBULA, AURORA, SUPERBOLIDE =
    16161, 25758, 16140, 16148, 36935, 16151, 16152

local S_NO_MERCY, S_READY_TO_BREAK, S_READY_TO_REIGN = 1831, 3886, 3840
local S_READY_TO_RIP, S_READY_TO_TEAR, S_READY_TO_GOUGE, S_READY_TO_BLAST, S_READY_TO_RAZE = 1842, 1843, 1844, 2686, 3839

local function max_cartridges(b)
    return b.level >= 88 and 3 or 2
end

local function gcd(b)
    if not b:in_melee() then
        if b:has(LIGHTNING_SHOT) then return LIGHTNING_SHOT end
        return nil
    end
    local carts = b:gauge_of("cartridges")
    local mercy = b:has_status(S_NO_MERCY)
    -- Chains under way.
    if b.combo == NOBLE_BLOOD then return LION_HEART end
    if b.combo == REIGN_OF_BEASTS then return NOBLE_BLOOD end
    -- Gnashing Fang's chain is its own (the gauge's ammo combo, not ActionCombo: it leaves the main combo be), so
    -- b.mem keeps its step.
    if b.mem.fang == 2 and b:has(WICKED_TALON) then
        b.mem.fang = nil
        return WICKED_TALON
    end
    if b.mem.fang == 1 and b:has(SAVAGE_CLAW) then
        b.mem.fang = 2
        return SAVAGE_CLAW
    end
    if b:has_status(S_READY_TO_REIGN) and b:has(REIGN_OF_BEASTS) then return REIGN_OF_BEASTS end
    if mercy and carts >= 2 and b:ready(DOUBLE_DOWN) then return DOUBLE_DOWN end
    if b:has(GNASHING_FANG) and b:ready(GNASHING_FANG) and carts >= 1
        and (mercy or b:cd(NO_MERCY) > 17 or not b:has(NO_MERCY)) then
        b.mem.fang = 1
        return GNASHING_FANG
    end
    if b:has_status(S_READY_TO_BREAK) and b:ready(SONIC_BREAK) and (mercy or b:status(S_READY_TO_BREAK) < 5) then
        return SONIC_BREAK
    end
    -- Cartridges spent in No Mercy, or before the combo's next would be lost.
    local spend = (b:aoe(3) and b:has(FATED_CIRCLE)) and FATED_CIRCLE or BURST_STRIKE
    local overflow = carts >= max_cartridges(b) and (b.combo == BRUTAL_SHELL or b.combo == DEMON_SLICE)
    if b:has(spend) and carts >= 1 and (mercy or overflow) then return spend end
    if b:aoe(3) and b:has(DEMON_SLICE) then
        if b.combo == DEMON_SLICE and b:has(DEMON_SLAUGHTER) then return DEMON_SLAUGHTER end
        return DEMON_SLICE
    end
    if b.combo == BRUTAL_SHELL and b:has(SOLID_BARREL) then return SOLID_BARREL end
    if b.combo == KEEN_EDGE and b:has(BRUTAL_SHELL) then return BRUTAL_SHELL end
    return KEEN_EDGE
end

local function ogcd(b)
    if not b:in_melee() then return nil end
    -- Mitigation when hurt.
    if b.hp < 25 and b:ready(SUPERBOLIDE) then return SUPERBOLIDE, "self" end
    local nebula = b:best(GREAT_NEBULA, NEBULA)
    if b.hp < 50 and nebula and b:ready(nebula) then return nebula, "self" end
    if b.hp < 65 and b:ready(CAMOUFLAGE) then return CAMOUFLAGE, "self" end
    local heart = b:best(HEART_OF_CORUNDUM, HEART_OF_STONE)
    if b.hp < 75 and heart and b:ready(heart) then return heart, "self" end
    if b.hp < 70 and b:ready(AURORA) then return AURORA, "self" end
    -- Continuations as they're ready.
    for status, action in pairs({ [S_READY_TO_RIP] = JUGULAR_RIP, [S_READY_TO_TEAR] = ABDOMEN_TEAR,
        [S_READY_TO_GOUGE] = EYE_GOUGE, [S_READY_TO_BLAST] = HYPERVELOCITY, [S_READY_TO_RAZE] = FATED_BRAND }) do
        if b:has_status(status) and b:ready(action) then return action end
    end
    local carts = b:gauge_of("cartridges")
    if b:ready(NO_MERCY) and (carts >= 1 or not b:has(BURST_STRIKE)) then return NO_MERCY, "self" end
    if b:ready(BLOODFEST) and carts == 0 then return BLOODFEST end
    local zone = b:best(BLASTING_ZONE, DANGER_ZONE)
    if zone and b:ready(zone) and (b:has_status(S_NO_MERCY) or b:cd(NO_MERCY) > 10) then return zone end
    if b:ready(BOW_SHOCK) and b:has_status(S_NO_MERCY) then return BOW_SHOCK, "self" end
    return nil
end

return {
    classjobs = { 37 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
