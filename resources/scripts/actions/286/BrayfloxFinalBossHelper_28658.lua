-- An action of Brayflox's Longstop's last boss's helpers (BNpcBase 9020) (territory 1041): a 9.0 s cast round a
-- puddle (10 yalms), 5.6 s after its 28657: 307 and status 2104 (a poison) for 15 s on whoever is still in it. From
-- Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the
-- Action sheet's.
local POTENCY = 100
local STATUS = 2104
local DURATION = 15.0
local TICK_POTENCY = 30

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:damage_over_time(STATUS, DURATION, magicalDamage(player, TICK_POTENCY))

    return effects
end
