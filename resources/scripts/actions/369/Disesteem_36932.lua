-- Disesteem (Lv. 100): 1,000 to the first enemy in a line, 25 % less to the rest; Scorn spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    drkDamage(effects, player, 1000)
    effects:consume_effect(DRK_EFFECT_SCORN)
    drkWeaponskill(effects, player)

    return effects
end
