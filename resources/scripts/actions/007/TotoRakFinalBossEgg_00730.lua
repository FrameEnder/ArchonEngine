-- The Thousand Maws of Toto-Rak's last boss's eggs (BNpcBase 572): a 2.7 s cast round itself 5 s after it lands, a hit
-- and status 2089 (a poison) for 15 s on those it reaches (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 116.5 to players, 99 to a Duty Support member, then
-- 9-11 a tick); the egg dies 2.1 s later (the director, content/f1r1_re.lua.) Its timeline's `recorded_amounts` scales
-- the hit, and the poison with it.
local POTENCY = 100
local STATUS = 2089
local DURATION = 15.0
local TICK_POTENCY = 10

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
