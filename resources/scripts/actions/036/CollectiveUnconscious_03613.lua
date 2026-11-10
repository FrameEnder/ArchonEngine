-- Collective Unconscious (Lv. 58): the party near takes 10 % less damage, 10 s (the regen while held isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(AST_EFFECT_COLLECTIVE_UNCONSCIOUS, 0, 10.0)

    return effects
end
