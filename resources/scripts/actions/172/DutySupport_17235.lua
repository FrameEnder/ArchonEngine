-- A Duty Support member's auto-attack (the Conjurer, who comes for a tank, `server/party_npc.rs`): NPC-only. What it
-- did on retail (1 a hit, Bob's Chronofoil capture c99fe374) is in the member's timeline
-- (resources/timelines/*SupportConjurer_<base>.json, `recorded_amounts`), which scales this hit to it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
