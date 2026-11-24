-- The Jade Stoa's leaper (10819): a 2.7 s cast where it came at the arena's north edge. From Bob's Chronofoil capture
-- 84700114 (2026-11-24, the Jade Stoa unsynced). Its area is the Action sheet's; the timeline's `recorded_amounts`
-- scale it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    return effects
end
