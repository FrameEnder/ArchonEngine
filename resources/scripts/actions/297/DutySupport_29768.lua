-- A Duty Support member's action (the third set's Marauder, `server/party_npc.rs`): NPC-only, no player has it. In its
-- kit since Brayflox's Longstop (2026-11-03) without a script: it did nothing. Its potency by what it did beside
-- 29299 (100): a median 233 to 29299's 54 (Bob's Chronofoil capture d1f99707 of 2026-11-17, five runs at 41: 114 and
-- 384 hits).

local POTENCY = 430

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
