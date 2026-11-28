-- The Ultimate Weapon's last battle's helper's 29330: no cast, from the spot of the lightning a 29329 left as the
-- player walks into it (content/questbattle/5061.lua): 708 and 744 to them the twice it was seen (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 50e72631.) Its area is the Action sheet's (4 yalms round the helper); the
-- timeline's `recorded_amounts` scale it.
local POTENCY = 300

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
