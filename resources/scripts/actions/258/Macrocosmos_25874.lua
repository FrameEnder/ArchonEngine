-- Macrocosmos (Lv. 90): 270 to the first enemy near, 40 % less to the rest; Macrocosmos (15 s) for the party.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astMagic(effects, player, 270, 40)
    effects:gain_effect(AST_EFFECT_MACROCOSMOS, 0, 15.0)

    return effects
end
