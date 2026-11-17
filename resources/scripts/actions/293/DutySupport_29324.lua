-- A Duty Support member's action (the Conjurer, who comes for a tank, `server/party_npc.rs`): NPC-only, no player has
-- it. What it did on retail is in the member's timeline (resources/timelines/*SupportConjurer_<base>.json,
-- `recorded_amounts`), which scales this hit to it; the potency only has to make some damage to scale. Retail's showed
-- no status with it (Aero's too: Bob's Chronofoil capture c99fe374.)

-- A member without a timeline (the third set) hits by the potency itself: in proportion to its filler's 100, by the
-- five Stone Vigil runs of Bob's capture d1f99707 (2026-11-17, at 41: its 469 to Stone II's 45 in the median.)
local POTENCY = 1040

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
