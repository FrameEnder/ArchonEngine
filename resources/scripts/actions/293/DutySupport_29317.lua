-- A Duty Support member's heal (the Conjurer's Cure, who comes for a tank, `server/party_npc.rs`): NPC-only, no player
-- has it. How much it healed on retail is in the member's timeline (resources/timelines/*SupportConjurer_<base>.json,
-- `recorded_amounts`), which sets this heal to it; the potency only has to make some healing.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(whmHeal(player, POTENCY))

    return effects
end
