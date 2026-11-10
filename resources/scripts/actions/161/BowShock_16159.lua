-- Bow Shock (Lv. 62): 150 to all enemies near and 60 over 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 150)
    effects:damage_over_time(GNB_EFFECT_BOW_SHOCK, 15.0, physicalDamage(player, 60))

    return effects
end
