-- The Praetorium's armour's action (Mount 17, from EObj 2012819): 1128, no cast, an area before it: 4500 to each
-- enemy it hit, every time (Bob's Chronofoil capture 6320ae56 of 2026-11-18, 39 uses; effect kind 3, value 4500). A
-- set amount, as retail's: nothing of the rider's goes into it.

local DAMAGE = 4500

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, DAMAGE)

    return effects
end
