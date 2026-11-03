-- An action of Brayflox's Longstop's 1044s (BNpcBase 1044) (territory 1041): a magic hit on its target every 7.4 s
-- (69 uses, 50-55 to the Duty Support tank). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run
-- with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
