-- Choco Strike (Defender skill 10): the chocobo companion's, an attack on all enemies in a cone before it, with more enmity.
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))
    effects:provoke()

    return effects
end
