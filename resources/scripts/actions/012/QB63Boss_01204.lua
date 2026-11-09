-- Quest battle 63's boss (BNpcBase 1988, 4270036): a 2.7 s cast at where its target stood, every 15-23 s (381 on the
-- player, 542-1126 on the ally; Connor's Chronofoil capture e834e436 of 2026-11-07, two runs), scaled to the
-- timeline's recorded amounts.
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
