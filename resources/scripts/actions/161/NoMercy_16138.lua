-- No Mercy (Lv. 2): 20 % more damage, 20 s; Ready to Break (30 s) from 54.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(GNB_EFFECT_NO_MERCY, 0, 20.0)
    if player.level >= 54 then
        effects:gain_effect_self(GNB_EFFECT_READY_TO_BREAK, 0, 30.0)
    end

    return effects
end
