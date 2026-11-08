-- History Repeating's quest battle (86): the area heal the one who talks (BNpcBase 1632) casts on its side (5.7 s,
-- aimed at one of them): 654 the median heal (630-978, the timeline's recorded amount) and Regen (158) on each it
-- reaches, ticking 38-58 every 3 s and last seen 17.1 s after it landed (Bob's Chronofoil captures of 2026-11-08).
local HEAL = 654
local REGEN = 158
local REGEN_SECONDS = 18.0
local REGEN_TICK = 40

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(HEAL)
    effects:heal_over_time(REGEN, REGEN_SECONDS, REGEN_TICK)

    return effects
end
