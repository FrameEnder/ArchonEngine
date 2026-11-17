-- A Duty Support member's action (the third set's Thaumaturge in the Stone Vigil, `server/party_npc.rs`): NPC-only, no
-- player has it. Its potency by what it did beside 29314 (100): a median 181 to 29314's 90 (Bob's Chronofoil capture
-- 216f28eb of 2026-11-17, seven runs at 41: 384 and 504 hits).

local POTENCY = 200

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
