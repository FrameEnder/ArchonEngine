-- The Praetorium's enemies' BNpcBase 14562's 29179: no cast, every 6.6 s (225 uses): 94 to a player, 61 to a member
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) Its area is the Action sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only
-- has to make some damage).

local POTENCY = 50

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
