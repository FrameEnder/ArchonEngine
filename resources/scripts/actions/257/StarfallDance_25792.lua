-- Starfall Dance (Lv. 90): a sure critical direct hit of 600 to the first enemy in a line, 75 % less to the rest; Flourishing Starfall spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    dncDamage(effects, player, 600)
    effects:consume_effect(DNC_EFFECT_FLOURISHING_STARFALL)

    return effects
end
