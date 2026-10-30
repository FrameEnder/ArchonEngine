-- The Thousand Maws of Toto-Rak's room bosses (BNpcBase 270) and the second one's adds (271): round themselves every
-- 6.4 s, no hit, status 2089 (a poison) for 15 s on everyone it reaches (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 57 a tick on the player, 19 on Duty
-- Support members; the adds' 3-9.) Their timelines' `recorded_amounts` set the tick.
local STATUS = 2089
local DURATION = 15.0
local TICK_POTENCY = 30

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
