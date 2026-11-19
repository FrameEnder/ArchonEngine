-- The Praetorium's second boss's 28483: a 4.7 s cast on its target: 1013 to a player, 733 to Duty Support's tank
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) The timeline's `recorded_amounts` scale it.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
