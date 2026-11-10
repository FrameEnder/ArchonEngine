-- Fountainfall (Lv. 40): 340; Silken (or Flourishing) Flow spent; a Feather half the time.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, tierPotency(player, { { 94, 340 }, { 1, 320 } }))
    dncProc(effects, player, DNC_EFFECT_SILKEN_FLOW, DNC_EFFECT_FLOURISHING_FLOW)

    return effects
end
