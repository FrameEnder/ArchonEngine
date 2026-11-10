-- Druochole (Lv. 45): heals 600; an Addersgall, 7 % of the MP back.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    sgeHeal(effects, player, 600)
    sgeAddersgall(effects, player)

    return effects
end
