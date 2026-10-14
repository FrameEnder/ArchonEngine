-- Chigoe. Hit about the same as its auto-attack on retail. TODO: it also heals the chigoe.
-- Potency is relative to the enemy auto-attack (100), from what Kawari Bestiary recorded on retail.
POTENCY = 95

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
