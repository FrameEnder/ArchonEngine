-- The Jade Stoa's Hakutei (10828): no cast: on the one he marked (87) 5.1 s before, and round them. From Bob's
-- Chronofoil capture 84700114 (2026-11-24, the Jade Stoa unsynced). Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    return effects
end
