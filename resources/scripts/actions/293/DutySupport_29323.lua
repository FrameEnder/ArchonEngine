-- A Duty Support member's action (the Conjurer, who comes for a tank, `server/party_npc.rs`): NPC-only, no player has
-- it. What it did on retail is in the member's timeline (resources/timelines/*SupportConjurer_<base>.json,
-- `recorded_amounts`), which scales this hit to it; the potency only has to make some damage to scale. Retail's showed
-- no status with it (Aero's too: Bob's Chronofoil capture c99fe374.)

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
