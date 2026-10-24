-- Choco Cure (Healer skill 4): the chocobo companion's, a heal (cure potency 300).
-- Potency from its Action description (7.57). Its damage is scaled to retail's by the companion's timeline
-- (`server/companion.rs`: Bob's Chronofoil capture of 2026-10-24, a level 88 companion with Attacker at 6.)
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, POTENCY))

    return effects
end
