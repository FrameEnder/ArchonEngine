-- Castrum Meridianum's helpers' 29269: no cast, on everyone, three in turn after the first boss's 29268: 286 to a
-- player, 248 to a member (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of
-- 2026-11-17, three runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
