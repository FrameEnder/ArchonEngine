-- Stellar Detonation (Lv. 62): the star: 205 to the enemies near and 540 healed, or 310 and 720 once it's a Giant Dominance (its last 10 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local left = player:effect_time_left(AST_EFFECT_EARTHLY_DOMINANCE) or 0
    local giant = left <= 10
    astMagic(effects, player, giant and 310 or 205)
    effects:heal_self(healAmount(player, levelPotency(player, giant and 720 or 540, LEVEL_MAIM_AND_MEND)))
    effects:consume_effect(AST_EFFECT_EARTHLY_DOMINANCE)

    return effects
end
