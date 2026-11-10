-- Provoke as NPCs use it (row 18: quest battle 60's gladiator ally on retail, Bob's runs of 2026-11-10): puts its user
-- at the top of the target's enmity, as the role action (7533) does.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:provoke()

    return effects
end
