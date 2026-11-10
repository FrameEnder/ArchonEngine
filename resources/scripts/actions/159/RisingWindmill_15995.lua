-- Rising Windmill (Lv. 35): 160 to all enemies near; Silken (or Flourishing) Symmetry spent; a Feather half the time.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, 160)
    dncProc(effects, player, DNC_EFFECT_SILKEN_SYMMETRY, DNC_EFFECT_FLOURISHING_SYMMETRY)

    return effects
end
