-- Wind Blade, Garuda-Egi's (and Chirada-Egi's and Suparna-Egi's) in Austerities of Wind (quest battle 81, BNpcBase
-- 2209, 2212, 2213): a 1 s cast, 5 yalms round where it lands (the Action sheet's), every 3.6-10.5 s, 58 on the player
-- and 79 on Y'mhitra (Bob's Kawari Recorder runs of 2026-11-12; their timelines' recorded amounts.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
