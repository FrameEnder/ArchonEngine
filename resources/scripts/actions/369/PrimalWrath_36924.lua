-- Primal Wrath (Lv. 96): 700 to the first enemy round the Warrior, 50 % less to the rest; only while Wrathful.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 700))
    effects:consume_effect(WAR_EFFECT_WRATHFUL)

    return effects
end
