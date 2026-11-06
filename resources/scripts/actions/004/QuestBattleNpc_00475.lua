-- An NPC action (On Holy Ground's three in Western Thanalan, BNpcBase 1243, and The Rematch's archers, 1242): a hit
-- on its target, blunt (Bob's Chronofoil capture accc33a9 of 2026-11-06: 8-24 to a level 25-34 gladiator, two of
-- nine missed.) How much it does comes from the NPC's timeline (recorded_amounts), not from this potency.
-- TODO: the potency isn't verified.
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
