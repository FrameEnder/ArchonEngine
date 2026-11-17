-- A Duty Support member's action (the third set's Lancer in the Stone Vigil, `server/party_npc.rs`): NPC-only, no
-- player has it. Its potency by what it did beside 29305 (100): a median 329 to 29305's 81 (Bob's Chronofoil capture
-- 216f28eb of 2026-11-17, seven runs at 41: 212 and 545 hits).

local POTENCY = 405

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))

    return effects
end
