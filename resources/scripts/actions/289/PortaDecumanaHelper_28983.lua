-- The Porta Decumana's helpers' 28983: a 2.7 s cast on three spots 15 yalms from the middle and the middle, or eight
-- 17 yalms out, with each of the boss's 28982s (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981
-- of 2026-11-17, one run each.) Its area is the Action sheet's; it hit no one in either run, so its damage here is a
-- guess.

local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
