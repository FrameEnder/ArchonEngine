-- Blasting Zone (Lv. 80): 800.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 800)

    return effects
end
