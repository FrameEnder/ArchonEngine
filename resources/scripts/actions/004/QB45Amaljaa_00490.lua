-- Lord of the Inferno's Amalj'aa lancers and soldiers (quest battle 45, BNpcBases 968 and 969): about every 10-12 s,
-- 81 on the ally it was after (Bob's Chronofoil capture b8432613, 2026-10-27, four runs: the median of the hits that
-- landed). Their timelines' `recorded_amounts` scale it to retail's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
