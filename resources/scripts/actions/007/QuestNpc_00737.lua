-- Lunge (BNpcBase 1904 in quest 910, 1747 in quest battle 61, 398 in Coerthas): a single hit, 60-330 on the player
-- (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06.) What it does to players and other NPCs there comes
-- from the NPC's timeline (recorded_amounts), not from this potency; it isn't verified for players who use it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
