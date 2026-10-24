-- Choco Surge (Healer skill 7): the chocobo companion's, 30% more healing magic potency for 30 s (Choco Surge, 238).
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local EFFECT_CHOCO_SURGE = 238
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_CHOCO_SURGE, 0, DURATION)

    return effects
end
