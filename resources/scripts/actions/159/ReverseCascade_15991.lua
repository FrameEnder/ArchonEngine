-- Reverse Cascade (Lv. 20): 280; Silken (or Flourishing) Symmetry spent; a Feather half the time.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, tierPotency(player, { { 94, 280 }, { 1, 260 } }))
    dncProc(effects, player, DNC_EFFECT_SILKEN_SYMMETRY, DNC_EFFECT_FLOURISHING_SYMMETRY)

    return effects
end
