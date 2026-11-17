-- Castrum Meridianum's enemies' 6500 (BNpcBase 14572): no cast seen; base 14572: 153 to players, 62 to npcs (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is
-- the Action sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only has to make some damage).

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
