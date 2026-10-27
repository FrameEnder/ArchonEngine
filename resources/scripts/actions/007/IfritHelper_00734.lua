-- Ifrit's helpers' Radiant Plume (the Bowl of Embers, BNpcBase 401): a 2.7 s cast where each was put, 8 yalms round
-- (the Action sheet's): four 8 yalms from the middle, then eight 20 yalms out. 181-227 to the player (Bob's Chronofoil capture b8432613 of 2026-10-27, the Bowl of Embers with Duty Support (visit 0.16)): its
-- timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
