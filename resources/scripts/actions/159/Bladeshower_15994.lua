-- Bladeshower (Lv. 25): 100 to all enemies near, 160 after Windmill with Silken Flow half the time; 5 Esprit from 76.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        dncDamage(effects, player, 160)
        dncChance(effects, DNC_EFFECT_SILKEN_FLOW, 0.5)
    else
        dncDamage(effects, player, 100)
    end
    dncEsprit(effects, player)

    return effects
end
