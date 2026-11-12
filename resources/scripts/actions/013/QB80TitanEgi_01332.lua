-- Earthen Fury, Titan-Egi's in Austerities of Earth (quest battle 80, BNpcBase 2203): a 6 s cast, 5 yalms round itself
-- (the Action sheet's, with its omen), once in Bob's Kawari Recorder run of 2026-11-12: 1243 on Y'mhitra, who stood in
-- it (its timeline's recorded amount.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
