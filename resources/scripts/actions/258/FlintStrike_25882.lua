-- Flint Strike (Lv. 60): 800 to the first enemy round the Monk, 35 % less to the rest; three different Beast Chakra; opens the Solar Nadi; Formless Fist.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(35)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 800))
    mnkBlitz(effects, player, 2)

    return effects
end
