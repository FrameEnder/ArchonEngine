-- Fountain (Lv. 2): 120, 280 after Cascade with Silken Flow half the time (30 s); 5 Esprit from 76.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        dncDamage(effects, player, 280)
        dncChance(effects, DNC_EFFECT_SILKEN_FLOW, 0.5)
    else
        dncDamage(effects, player, tierPotency(player, { { 94, 120 }, { 1, 100 } }))
    end
    dncEsprit(effects, player)

    return effects
end
