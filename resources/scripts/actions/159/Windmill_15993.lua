-- Windmill (Lv. 15): 120 to all enemies near; Silken Symmetry half the time; 5 Esprit from 76.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, 120)
    dncChance(effects, DNC_EFFECT_SILKEN_SYMMETRY, 0.5)
    dncEsprit(effects, player)

    return effects
end
