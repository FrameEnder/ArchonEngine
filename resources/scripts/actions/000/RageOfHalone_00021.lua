-- Used by the handsome stranger in Underneath the Sultantree's quest battle. What it does to NPCs there comes from Bob's
-- Chronofoil capture of retail (the NPC's timeline, recorded_amounts), not from this potency.
-- TODO: the potency (and its combo bonus after Riot Blade) isn't verified for players who use it.
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
