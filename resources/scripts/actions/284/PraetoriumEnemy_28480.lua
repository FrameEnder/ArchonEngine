-- The Praetorium's second boss's BNpcBase 14452's 28480: no cast, on a player, with a knockback: 399 to a player, 310
-- to a member (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of
-- 2026-11-17 and 18, five runs.) The timeline's `recorded_amounts` scale it. The knockback isn't done: its row wasn't
-- read.

local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
