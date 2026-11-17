-- The Porta Decumana's second boss's 29022: a 4.7 s cast on itself: 907 to 1046 to the players and 269 to 995 to the
-- members, everyone (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run
-- each.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
