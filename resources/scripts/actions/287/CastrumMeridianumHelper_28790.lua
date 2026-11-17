-- Castrum Meridianum's helpers' 28790: a 4 s cast from six spots: 1160 to a player (Connor's Chronofoil capture
-- af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is the Action sheet's;
-- the timeline's `recorded_amounts` scale it.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
