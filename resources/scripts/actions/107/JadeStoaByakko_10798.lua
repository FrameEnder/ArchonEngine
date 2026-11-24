-- The Jade Stoa's Byakko (10798): a 3.7 s cast round himself, knocking back whoever it hits (the Knockback sheet's
-- row 70, all five hits). From Bob's Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced). Its area is
-- the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 100
local KNOCKBACK = 70

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:knockback(KNOCKBACK)
    return effects
end
