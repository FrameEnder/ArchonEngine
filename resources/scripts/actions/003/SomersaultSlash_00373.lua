-- Ground Squirrel. Hit about 1.7x its auto-attack on retail (one sample).
-- Potency is relative to the enemy auto-attack (100), from what Kawari Bestiary recorded on retail.
POTENCY = 170

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
