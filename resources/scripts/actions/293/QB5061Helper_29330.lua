-- The Ultimate Weapon's last battle's helper's 29330: no cast, from where 29329 was cast, 3.7 s after it landed: 708
-- to the player the once it caught one (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of
-- 2026-11-17, two runs.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
