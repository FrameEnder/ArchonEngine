-- Divination (Lv. 50): 6 % more damage for the party near, 20 s; Divining (30 s) from 92.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_DIVINATION, 0, 20.0)
    if player.level >= 92 then
        effects:gain_effect_self(AST_EFFECT_DIVINING, 0, 30.0)
    end

    return effects
end
