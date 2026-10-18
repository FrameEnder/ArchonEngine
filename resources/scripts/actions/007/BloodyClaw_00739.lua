-- Used by the blanga in Underneath the Sultantree's quest battle. What it does to other NPCs there comes from Bob's
-- Chronofoil capture of retail (the NPC's timeline, recorded_amounts), not from this potency.
-- TODO: the potency (and any status it gives) isn't verified.
POTENCY = 150

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
