-- Shadow Of Death (Lv. 10): 300; Death's Design (10 % more damage from the Reaper; here from anyone), 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 300)
    effects:gain_effect(RPR_EFFECT_DEATHS_DESIGN, 0, 30.0)

    return effects
end
