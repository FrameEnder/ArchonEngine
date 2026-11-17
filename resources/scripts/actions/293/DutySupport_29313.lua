-- A Duty Support member's action (the third set's Thaumaturge in the Stone Vigil, `server/party_npc.rs`): NPC-only, no
-- player has it. Its potency by what it did beside 29314 (100): a median 963 to 29314's 90 (Bob's Chronofoil capture
-- 216f28eb of 2026-11-17, seven runs at 41: 426 and 504 hits). On retail it hit up to three enemies (two 68 times and
-- three 14 of 330); here its target alone.

local POTENCY = 1070

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
