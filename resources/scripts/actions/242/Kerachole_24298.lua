-- Kerachole (Lv. 50): the party near takes 10 % less damage and 100 over 15 s (78), 15 s; an Addersgall.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SGE_EFFECT_KERACHOLE, 0, 15.0)
    if player.level >= 78 then
        effects:heal_over_time(SGE_EFFECT_KERAKEIA, 15.0, healAmount(player, levelPotency(player, 100, LEVEL_MAIM_AND_MEND)))
    end
    sgeAddersgall(effects, player)

    return effects
end
