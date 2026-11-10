-- Celestial Opposition (Lv. 60): heals the party near 200 (its 100 over 15 s isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 200)

    return effects
end
