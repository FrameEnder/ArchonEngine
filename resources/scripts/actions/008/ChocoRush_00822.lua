-- Choco Rush (Attacker skill 7): the chocobo companion's, an attack that stuns for 5 s.
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local POTENCY = 130
local STUN_DURATION = 5.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))
    effects:gain_effect(EFFECT_STUN, 0, STUN_DURATION)

    return effects
end
