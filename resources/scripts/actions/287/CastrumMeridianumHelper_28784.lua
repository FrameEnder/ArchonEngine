-- Castrum Meridianum's helpers' 28784: a 3 s cast under each of the party (a circle of 5): 1228 to a member (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is
-- the Action sheet's; the timeline's `recorded_amounts` scale it (1298 to a player: Bob's White Mage, Chronofoil
-- capture deea324c of 2026-11-19).

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
