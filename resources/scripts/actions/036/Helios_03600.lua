-- Helios (Lv. 10): heals the Astrologian and the party near 400.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 400)

    return effects
end
