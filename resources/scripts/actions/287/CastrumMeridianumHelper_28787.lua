-- Castrum Meridianum's helpers' 28787: a 7 s cast from eight spots round the last boss's room (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is the Action
-- sheet's; it hit no one in the runs, so its damage here is a guess.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
