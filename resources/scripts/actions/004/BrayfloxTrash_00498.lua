-- An action of Brayflox's Longstop's 1047 (BNpcBase 1047) (territory 1041): a gaze (4.0 s, once); it caught no one
-- in the run, so what it does to whoever it catches isn't recorded: a hit at the potency below until it is. From
-- Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the
-- Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gaze()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
