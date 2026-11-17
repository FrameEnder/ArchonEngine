-- The Porta Decumana's second boss's 29024: the last part's 70.7 s cast on itself. It never landed: the boss fell 54
-- and 63 s into it (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run
-- each.) 9999999 to everyone here, a guess.

local DAMAGE = 9999999

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, DAMAGE)

    return effects
end
