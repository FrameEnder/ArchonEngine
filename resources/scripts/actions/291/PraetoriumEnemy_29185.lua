-- The Praetorium's enemies' BNpcBase 14566's 29185: no cast, every 2.1 s, hit three at once: 612 to a player, 496 to
-- a member (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of
-- 2026-11-17 and 18, five runs.) Its area is the Action sheet's; the enemies' timelines' `recorded_amounts` scale it
-- (the potency only has to make some damage).

local POTENCY = 200

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
