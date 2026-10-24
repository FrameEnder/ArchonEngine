-- Choco Medica (Healer skill 10): the chocobo companion's, a heal on itself and nearby party members (cure potency 150).
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, POTENCY))

    return effects
end
