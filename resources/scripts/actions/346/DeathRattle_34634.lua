-- Death Rattle (Lv. 55): 280 after a finisher.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 280)

    return effects
end
