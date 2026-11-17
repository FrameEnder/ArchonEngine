-- The Ultimate Weapon's last battle's orbs' 29333: a 2.7 s cast where each stands, 7.7 s after they come; it hit no
-- one in either run (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.)
-- Its area is the Action sheet's; its damage here is a guess.
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
