-- Elixir Burst (Lv. 92): 900 to the first enemy round the Monk, 35 % less to the rest; three of one Beast Chakra; opens the Lunar Nadi; Formless Fist.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(35)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 900))
    mnkBlitz(effects, player, 1)

    return effects
end
