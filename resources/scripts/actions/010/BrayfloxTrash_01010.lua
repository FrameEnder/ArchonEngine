-- An action of Brayflox's Longstop's 1047 (BNpcBase 1047) (territory 1041): a hit round itself (once, 61 to the
-- tank). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area and
-- cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
