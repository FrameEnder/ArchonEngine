-- Quest battle 64's ally (BNpcBase 1876, 4277164): 72 on an enemy, once a run (Connor's Chronofoil capture e834e436 of
-- 2026-11-07), scaled to the timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
