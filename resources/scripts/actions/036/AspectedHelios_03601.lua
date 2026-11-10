-- Aspected Helios (Lv. 40): heals the party near 250 and 150 over 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 250)
    effects:heal_over_time(AST_EFFECT_ASPECTED_HELIOS, 15.0, healAmount(player, levelPotency(player, 150, LEVEL_MAIM_AND_MEND)))

    return effects
end
