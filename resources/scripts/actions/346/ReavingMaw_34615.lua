-- Reaving Maw (Lv. 35): 120 to all enemies near, 140 with Honed Reavers; Honed Steel.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 120 + vprBonus(effects, player, VPR_EFFECT_HONED_REAVERS, 20))
    effects:gain_effect_self(VPR_EFFECT_HONED_STEEL, 0, 60.0)

    return effects
end
