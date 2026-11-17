-- A Duty Support member's action (the tank in Sastasha, `server/party_npc.rs`): NPC-only, no player has it.
-- What it did on retail is in the member's timeline (resources/timelines/DutySupport*_<base>.json, `recorded_amounts`),
-- which scales this hit to it; the potency only has to make some damage to scale.

-- A member without a timeline (the third set) hits by the potency itself: in proportion to its filler's 100, by the
-- five Stone Vigil runs of Bob's capture d1f99707 (2026-11-17, at 41: its 270 to 29299's 54 in the median.)
local POTENCY = 500

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
