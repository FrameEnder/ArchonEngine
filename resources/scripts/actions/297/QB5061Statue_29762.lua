-- The Ultimate Weapon's last battle's four of base 14697's 29762: a 3.7 s cast from the arena's edge, facing the
-- middle: 1016 and 1022 to the player the once two caught them (Connor's Chronofoil capture af05916c of 2026-11-09
-- and Bob's 9ba7d981 of 2026-11-17, two runs.) Its area is the Action sheet's; the timeline's `recorded_amounts`
-- scale it.
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
