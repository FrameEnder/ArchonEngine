-- The Jade Stoa's Byakko (10796): a 3.7 s cast from the middle (facing one of the party) or an edge or corner (facing
-- the middle). From Bob's Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced). Its area is the Action
-- sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    return effects
end
