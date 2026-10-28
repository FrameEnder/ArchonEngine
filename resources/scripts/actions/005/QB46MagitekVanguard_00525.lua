-- The magitek vanguard's Drill Cannons (quest battle 46, BNpcBase 1016): a 2.0 s cast, a 30-yalm line the way it faces
-- (the Action sheet's area). 186.5 (median) to the Twin Adders in Bob's Chronofoil capture b8432613 of 2026-10-27; its
-- timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, physicalDamage(player, POTENCY))

    return effects
end
