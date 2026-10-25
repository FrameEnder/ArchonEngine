-- The Copperbell Mines' ambushing gigas (BNpcBase 1398, 1403): a hit every 10-12 s, sometimes on two (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: 48-60 to
-- players.) Their timelines' `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
