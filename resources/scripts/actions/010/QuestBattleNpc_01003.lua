-- An action quest battle NPCs use (Nouveau Riche's gladiator, BNpcBase 1645): a hit, 306 on the gladiator on your side
-- (Bob's Chronofoil capture caa5e908, 2026-10-30: its effect, with a second of kind 31 not understood). Its timeline's
-- `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
