-- Chigoe. Hit about the same as its auto-attack on retail, and gives the chigoe back what it took.
-- Potency is relative to the enemy auto-attack (100), from what Kawari Bestiary recorded on retail.
POTENCY = 95

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local damage = player.parameters:calc_physical_damage(POTENCY)
    effects:damage(DAMAGE_TYPE_PIERCING, damage)
    -- Against players this becomes whatever the damage came to (see action.rs.)
    effects:heal_self(damage)

    return effects
end
