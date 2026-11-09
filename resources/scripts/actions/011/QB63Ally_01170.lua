-- Quest battle 63's ally (BNpcBase 1987, 4270049): its weapon skill, 76-95 on an enemy every 4-7 s (Connor's
-- Chronofoil capture e834e436 of 2026-11-07, two runs), scaled to the timeline's recorded amount.
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
