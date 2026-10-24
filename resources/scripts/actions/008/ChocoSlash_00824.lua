-- Choco Slash (Attacker skill 1): the chocobo companion's, an attack.
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
-- Retail: blunt (0x73).
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
