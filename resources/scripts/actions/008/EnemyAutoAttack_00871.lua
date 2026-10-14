-- Enemy auto-attack (the piercing kind, used by chigoes, zizes and Wood Wailers).
-- Potency is relative to the enemy auto-attack (100), from what Kawari Bestiary recorded on retail.
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
