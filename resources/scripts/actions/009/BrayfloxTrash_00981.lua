-- An action of Brayflox's Longstop's 1048s (BNpcBase 1048) (territory 1041): a magic hit every 8.9 s (15 uses, 51-55
-- to the tank). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area
-- and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
