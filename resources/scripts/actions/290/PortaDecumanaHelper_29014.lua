-- The Porta Decumana's helpers' 29014: a 4.7 s cast on one of the party with the second boss's 29013: 829 to 948 to a
-- player and 310 to 917 to a member, all four each time (they stood together: whether it's shared between them isn't
-- known) (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) Its
-- area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
