-- A Duty Support member's action (the Lancer in Sastasha, `server/party_npc.rs`): NPC-only, no player has it.
-- What it did on retail is in the member's timeline (resources/timelines/DutySupport*_<base>.json, `recorded_amounts`),
-- which scales this hit to it; the potency only has to make some damage to scale.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))

    return effects
end
