-- The Ultimate Weapon's last battle's boss's 29342: no cast, on the player, 1.9 s after 29341 in the second part: 811
-- to 1033 (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.)
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
