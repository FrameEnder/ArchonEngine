-- An action of Brayflox's Longstop's goblin (BNpcBase 422, on the party's side) (territory 1041): a hit on the enemy
-- it fights every 16.1 s (1 each time). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20: one run with
-- Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
