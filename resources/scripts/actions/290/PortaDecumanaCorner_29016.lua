-- The Porta Decumana's base 14594's 29016: no cast, on the member of the party it reached: 159 to 195, and it's gone
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) The timeline's
-- `recorded_amounts` scale it.

local POTENCY = 80

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
