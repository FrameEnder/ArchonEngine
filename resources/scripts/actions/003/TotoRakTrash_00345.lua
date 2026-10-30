-- The Thousand Maws of Toto-Rak's BNpcBase 1360: a 2.2 s cast every 20.8 s that hits nothing and poisons its target,
-- status 18 for 15 s (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 48 a tick on players, the median.) Its timeline's `recorded_amounts` sets the tick.
local STATUS = 18
local DURATION = 15.0
local TICK_POTENCY = 30

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
