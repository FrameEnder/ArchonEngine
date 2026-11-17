-- The Porta Decumana's base 14589's 29000: a 2.7 s cast from the same edge spot 2.7 s after its 28999 lands, turned a
-- little (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) Its
-- area is the Action sheet's; it hit no one in either run, so its damage here is a guess.

local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
