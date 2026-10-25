-- The Copperbell Mines' last boss's auto-attack (BNpcBase 14537), every 3.0 s (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 39 to players.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
