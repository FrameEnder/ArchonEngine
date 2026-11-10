-- Exaltation (Lv. 86): 10 % less damage taken, 8 s, and 500 healed at its end (here at once; the reduction isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 500)

    return effects
end
