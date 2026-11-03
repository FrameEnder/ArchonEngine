-- An action of Brayflox's Longstop's second boss's add (BNpcBase 991) (territory 1041): a hit on its target (30-31
-- to the tank) and status 17 for 15 s (7 s once). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20:
-- one run with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(17, 0, 15.0)

    return effects
end
