-- An action of Brayflox's Longstop's third boss (BNpcBase 427) (territory 1041): a magic hit round its target, up to
-- three caught (116-451 to the Duty Support members), about every 21 s. From Bob's Chronofoil capture 44df02fc
-- (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
