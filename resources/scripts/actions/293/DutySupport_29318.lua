-- A Duty Support member's action (the fourth set's Conjurer at 50, `server/party_npc.rs`): NPC-only, no player has
-- it. What it did on retail is in the member's timeline (resources/timelines/FourthSupport*_<base>.json,
-- `recorded_amounts`), which scales it; the potency is its share beside the member's filler (100) in the Porta
-- Decumana (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) Its
-- heal on the lowest of the party: 1654 to 1732.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(whmHeal(player, POTENCY))

    return effects
end
