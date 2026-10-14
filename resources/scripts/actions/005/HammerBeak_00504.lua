-- Brood Ziz. Hit about 2.2x its auto-attack on retail.
-- Potency is relative to the enemy auto-attack (100), from what Kawari Bestiary recorded on retail.
POTENCY = 220

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
