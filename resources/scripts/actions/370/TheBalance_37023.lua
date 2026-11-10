-- The Balance (Lv. 30): 6 % more damage for a melee or tank, 3 % otherwise (here 6 %), 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_THE_BALANCE, 0, 15.0)
    astPlay(effects, player, 1)

    return effects
end
