-- Diagnosis (Lv. 2): heals 450.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 450, true)

    return effects
end
