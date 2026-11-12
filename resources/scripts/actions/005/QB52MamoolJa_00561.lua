-- Spinning Kick, the second wave's Mamool Ja mercenaries' in Pincer Maneuver (quest battle 52, BNpcBase 2065): a 3 s
-- cast, 3 yalms round it (the Action sheet's), twice in Bob's Kawari Recorder run of 2026-11-12, 45 on Aersthota each
-- time (its timeline's recorded amount.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
