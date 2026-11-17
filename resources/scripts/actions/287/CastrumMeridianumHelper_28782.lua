-- Castrum Meridianum's helpers' 28782: a 5 s cast from the middle of the second boss's room (a ring from 12.5 yalms):
-- 1288 to a player (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17,
-- three runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 500

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
