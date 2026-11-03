-- An action of Brayflox's Longstop's second boss (BNpcBase 990) (territory 1041): a fire attack round itself, up to
-- four caught (132-416 to the Duty Support members), every 30.8 s once its add has fallen. From Bob's Chronofoil
-- capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
