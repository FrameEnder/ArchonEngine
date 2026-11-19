-- The Praetorium's second boss's 28482: a 2.7 s cast before itself (a cone of 7 yalms, 120 degrees) (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18, five
-- runs.) Its area is the Action sheet's; it hit no one in the runs, so its damage here is a guess.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
