-- The Porta Decumana's base 14591's 29003: a 3.7 s cast from the first arena's middle: 533 to 540 to the players and
-- 215 to 562 to the members, and everyone thrown to the edge (the Knockback sheet's row 67; effect type 31 with it:
-- the members landed 17 yalms from the middle) (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981
-- of 2026-11-17, one run each.) Its area is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 250
local KNOCKBACK = 67

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
