-- Used by the masked mage in a quest battle. What it does to players and other NPCs there comes from Kawari Recorder's
-- recording of retail (the NPC's timeline, recorded_amounts), not from this potency.
-- TODO: the potency (and any status it gives) isn't verified for players who use it.
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
