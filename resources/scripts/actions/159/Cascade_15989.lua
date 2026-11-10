-- Cascade (Lv. 1): 220; Silken Symmetry half the time (30 s); 5 Esprit from 76.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, tierPotency(player, { { 94, 220 }, { 1, 200 } }))
    dncChance(effects, DNC_EFFECT_SILKEN_SYMMETRY, 0.5)
    dncEsprit(effects, player)

    return effects
end
