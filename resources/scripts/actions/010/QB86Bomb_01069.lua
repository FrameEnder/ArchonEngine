-- History Repeating's quest battle (86): the bombs' blast (BNpcBase 1633 and 1634), on the player they reached. A
-- magic hit (230 the median of 23 on retail, scaled to the timeline's recorded amount); the bomb falls 1.0 s later,
-- which the director does (content/questbattle/86.lua). From Bob's Chronofoil captures of 2026-11-08.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
