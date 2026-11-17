-- The Ultimate Weapon's last battle's helper's 29758: a 4.7 s cast from the boss's spot with his 29757: 192 to 248 to
-- the player (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) Its
-- area is the Action sheet's; the timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
