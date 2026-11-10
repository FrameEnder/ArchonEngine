-- Prognosis (Lv. 10): heals the Sage and the party near 300.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 300, true)

    return effects
end
