-- Lightspeed (Lv. 6): its spells instant, 15 s (`INSTANT_CASTS`).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(AST_EFFECT_LIGHTSPEED, 0, 15.0)

    return effects
end
