-- An action of Brayflox's Longstop's dragon (BNpcBase 429) (territory 1041): as it lands: everyone round it hit (127
-- to the Duty Support members, 157 to the player). From Bob's Chronofoil capture 44df02fc (2026-11-03, visit 0.20:
-- one run with Duty Support). Its area and cast are the Action sheet's.
-- It knocks back: the Knockback sheet's row 1, after the damage (9 of the 10 it hit in that run).
local POTENCY = 100
local KNOCKBACK = 1

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
