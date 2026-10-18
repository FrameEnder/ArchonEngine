-- Used by the handsome stranger in Underneath the Sultantree's quest battle. What it does to NPCs there comes from Bob's
-- Chronofoil capture of retail (the NPC's timeline, recorded_amounts), not from this potency.
-- TODO: the potency isn't verified for players who use it.
POTENCY = 270

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
