-- Celestial Intersection (Lv. 74): heals 200 and a barrier of twice it, 30 s (barriers don't absorb yet).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 200)

    return effects
end
