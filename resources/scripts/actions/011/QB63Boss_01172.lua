-- Quest battle 63's boss (BNpcBase 1988, 4270036): its swing every 2 s, in place of an auto-attack (95-117 on the
-- player; Connor's Chronofoil capture e834e436 of 2026-11-07, two runs), scaled to the timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
