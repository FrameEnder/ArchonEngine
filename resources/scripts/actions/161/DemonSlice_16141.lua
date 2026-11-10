-- Demon Slice (Lv. 10): 100 to all enemies near.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 100)

    return effects
end
