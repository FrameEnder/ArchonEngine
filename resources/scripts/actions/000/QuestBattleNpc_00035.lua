-- An action of the old marauder's kit that quest battle NPCs still use (Skull Sunder, the Maelstrom's lead in The
-- Company You Keep's battle, BNpcBase 1030): NPC-only now. What it did on retail (104, Bob's Chronofoil capture
-- cc62ad94) is in the NPC's timeline (`recorded_amounts`), which scales this hit to it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
