-- Castrum Meridianum's enemies' 6497 (BNpcBase 14568, 14569, 14575, 14664): no cast seen; base 14568: 105 to players,
-- 37 to npcs; base 14569: 170 to players, 61 to npcs; base 14575: 103 to players, 37 to npcs; base 14664: 213 to
-- players, 74 to npcs (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of
-- 2026-11-17, three runs.) Its area is the Action sheet's; the enemies' timelines' `recorded_amounts` scale it (the
-- potency only has to make some damage).

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
