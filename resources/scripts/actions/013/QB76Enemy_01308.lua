-- Heart of the Forest's second wave's 1308 (quest battle 76, BNpcBase 1703): instant, round itself, no hit: status 18
-- (a poison) on everyone it reaches but the one who prays, its tick 89 on the three who guard and 15 on the player
-- (the ActionEffect's own numbers, the timeline's recorded amounts); the guards lost it 32 s later (30 s here.) From
-- Bob's Chronofoil capture 667a68c5 (2026-11-16, three runs, ten uses). Its area is the Action sheet's.
local STATUS = 18
local DURATION = 30.0
local TICK_POTENCY = 30

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
