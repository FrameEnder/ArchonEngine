-- Higanbana (Lv. 30): 200 and 50 over 60 s; one Sen; a Meditation stack.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samDamage(effects, player, 200)
    effects:damage_over_time(SAM_EFFECT_HIGANBANA, 60.0, physicalDamage(player, 50))
    samIaijutsu(effects, player, nil)

    return effects
end
