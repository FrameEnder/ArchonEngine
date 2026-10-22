-- The NPC version of Fast Blade, used by Victory in Peril's corsair marauder (3970684) on the allied marauder. What
-- it does there comes from Bob's Chronofoil capture of retail (the NPC's timeline, recorded_amounts: 39), not from
-- this potency.
-- TODO: the potency isn't verified.
POTENCY = 150

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
