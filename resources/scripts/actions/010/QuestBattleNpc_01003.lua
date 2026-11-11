-- An action quest battle NPCs use (Nouveau Riche's gladiator, BNpcBase 1645): a hit, 306 on the gladiator on your side
-- (Bob's Chronofoil capture caa5e908, 2026-10-30), and a knockback, the Knockback sheet's row 19, after it (kind 31, on 11
-- of the 12 it hit in Bob's captures). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100
local KNOCKBACK = 19

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
