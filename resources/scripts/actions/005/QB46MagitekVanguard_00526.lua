-- The magitek vanguard's Cermet Drill (quest battle 46, BNpcBase 1016): a 2.5 s cast, a 6-yalm line the way it faces
-- (the Action sheet's area). 145 to the lead gladiator in Bob's Chronofoil capture b8432613 of 2026-10-27 (one hit
-- seen); its timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, physicalDamage(player, POTENCY))

    return effects
end
