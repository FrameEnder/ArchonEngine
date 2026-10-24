-- Choco Beak (Attacker skill 4): the chocobo companion's, an attack and damage over time (potency 20, 18 s).
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
-- Retail: piercing (0x72), Choco Beak (236) on the target with every use, about every 8 s.
local POTENCY = 130
local DOT_POTENCY = 20
local DURATION = 18.0
local EFFECT_CHOCO_BEAK = 236

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))
    effects:damage_over_time(EFFECT_CHOCO_BEAK, DURATION, physicalDamage(player, DOT_POTENCY))

    return effects
end
