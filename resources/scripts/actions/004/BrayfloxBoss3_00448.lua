-- An action of Brayflox's Longstop's third boss (BNpcBase 427) (territory 1041): a 2.5 s cast in front of it, every
-- 45 s (three uses); it caught no one in the run, so what it does to whoever it catches isn't recorded: a hit at the
-- potency below until it is. From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty
-- Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
