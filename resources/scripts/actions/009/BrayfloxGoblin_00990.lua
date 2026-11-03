-- An action of Brayflox's Longstop's goblin (BNpcBase 422, on the party's side) (territory 1041): an area round its
-- target (up to two enemies caught, 1 each) and status 197 (param 100) for 20 s on them. From Bob's Chronofoil
-- capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(197, 100, 20.0)

    return effects
end
