-- Helios Conjunction (Lv. 96): heals the party near 250 and 175 over 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 250)
    effects:heal_over_time(AST_EFFECT_HELIOS_CONJUNCTION, 15.0, healAmount(player, levelPotency(player, 175, LEVEL_MAIM_AND_MEND)))

    return effects
end
