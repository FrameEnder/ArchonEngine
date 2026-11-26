-- The Praetorium's armour's other action (Mount 17, from EObj 2012819): 1129, no cast, a line from the rider (the Action
-- sheet's CastType 4, 10 long, XAxisModifier 2 wide), at itself: 3000 to each enemy it hit, every time (Bob's Chronofoil
-- capture 35e30205 of 2026-11-25, 7 hits in 17 uses; effect kind 3, value 3000, as 1128's 4500). A set amount, as
-- retail's: nothing of the rider's goes into it.

local DAMAGE = 3000

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, DAMAGE)

    return effects
end
