-- Quest battle 62's boss (BNpcBase 1652, 4264130): a 1.7 s cast on one target, every 18 s (316 on the player,
-- 384-623 on an ally; Connor's Chronofoil capture e834e436 and Bob's 689b15da), scaled to the timeline's recorded
-- amounts.
local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
