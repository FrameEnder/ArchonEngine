-- The Thousand Maws of Toto-Rak's BNpcBase 1355: every 28.8 s, no hit, status 18 (a poison) for 15 s on those it
-- reaches (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 20-30 a tick on players, 10 on Duty Support members.) Its timeline's `recorded_amounts` sets the tick.
local STATUS = 18
local DURATION = 15.0
local TICK_POTENCY = 30

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
