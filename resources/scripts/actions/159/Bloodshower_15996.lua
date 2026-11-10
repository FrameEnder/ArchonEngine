-- Bloodshower (Lv. 45): 200 to all enemies near; Silken (or Flourishing) Flow spent; a Feather half the time.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, 200)
    dncProc(effects, player, DNC_EFFECT_SILKEN_FLOW, DNC_EFFECT_FLOURISHING_FLOW)

    return effects
end
