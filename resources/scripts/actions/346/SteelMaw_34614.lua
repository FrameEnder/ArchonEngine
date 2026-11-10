-- Steel Maw (Lv. 25): 120 to all enemies near, 140 with Honed Steel; Honed Reavers.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 120 + vprBonus(effects, player, VPR_EFFECT_HONED_STEEL, 20))
    effects:gain_effect_self(VPR_EFFECT_HONED_REAVERS, 0, 60.0)

    return effects
end
