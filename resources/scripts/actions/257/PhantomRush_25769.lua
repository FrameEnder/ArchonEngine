-- Phantom Rush (Lv. 90): 1,500 to the target, 35 % less round it; both Nadi and three Beast Chakra; Formless Fist.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(35)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 1500))
    mnkBlitz(effects, player, 3)

    return effects
end
