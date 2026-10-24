-- Choco Regen (Healer skill 1): the chocobo companion's, Regen on its target (cure potency 25, 18 s; Choco Regen, 237).
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local POTENCY = 25
local DURATION = 18.0
local EFFECT_CHOCO_REGEN = 237

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(EFFECT_CHOCO_REGEN, DURATION, healAmount(player, POTENCY))

    return effects
end
