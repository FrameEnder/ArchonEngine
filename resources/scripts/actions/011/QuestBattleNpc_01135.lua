-- Rive as quest battle 60's marauder uses it (BNpcBase 1731): an area attack (the Action sheet's) on two at a time, 222
-- on the NPCs it caught (the wyvern's hits did nothing; Bob's Chronofoil capture 21f967d2 of 2026-11-07, three uses:
-- the first run in which the marauder lived long enough.) What it does there comes from the NPC's timeline
-- (recorded_amounts), not from this potency; it isn't verified for players who use it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
