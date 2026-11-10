-- Ixochole (Lv. 52): heals the party near 400; an Addersgall.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 400)
    sgeAddersgall(effects, player)

    return effects
end
