-- Gluttony (Lv. 76): 560 to the target, 25 % less round it; 50 Soul; two Soul Reaver (Executioner from 96), 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    rprDamage(effects, player, 560, true)
    effects:gauge_add("soul", -50)
    if player.level >= 96 then
        effects:gain_effect_self(RPR_EFFECT_EXECUTIONER, 2, 30.0)
    else
        effects:gain_effect_self(RPR_EFFECT_SOUL_REAVER, 2, 30.0)
    end

    return effects
end
