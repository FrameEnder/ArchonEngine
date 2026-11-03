-- An action of Brayflox's Longstop's last boss (BNpcBase 14533) (territory 1041): a 3.0 s breath in front of it
-- every 48.2 s: 385-400 to the player and status 2104 (a poison) for 15 s. From Bob's Chronofoil capture 44df02fc
-- (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the Action sheet's.
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
