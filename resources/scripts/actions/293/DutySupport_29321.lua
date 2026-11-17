-- A Duty Support member's action (the fourth set's Conjurer at 50, `server/party_npc.rs`): NPC-only, no player has
-- it. What it did on retail is in the member's timeline (resources/timelines/FourthSupport*_<base>.json,
-- `recorded_amounts`), which scales it; the potency is its share beside the member's filler (100) in the Porta
-- Decumana (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) A
-- heal for everyone round it, with most of them hurt: 510 to 541 each, and status 150 (param 59) on each; what the
-- status healed after wasn't measured (not done: the status only.)

local POTENCY = 140
local STATUS = 150
local STATUS_PARAM = 59
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(whmHeal(player, POTENCY))
    effects:gain_effect(STATUS, STATUS_PARAM, DURATION)

    return effects
end
