-- Choco Kick (Defender skill 4): the chocobo companion's, an attack with Slow (+20%, 12 s).
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local POTENCY = 130
local EFFECT_SLOW = 9
local SLOW_PERCENT = 20
local SLOW_DURATION = 12.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))
    effects:gain_effect(EFFECT_SLOW, SLOW_PERCENT, SLOW_DURATION)

    return effects
end
