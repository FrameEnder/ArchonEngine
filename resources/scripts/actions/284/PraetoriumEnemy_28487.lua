-- The Praetorium's last boss's fight, BNpcBase 14456's: a 2.7 s cast, three in the middle the way the boss faces (a
-- rectangle 40 by 4): 1072 to a member (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981,
-- 50e72631 and 6320ae56 of 2026-11-17 and 18, five runs.) Its area is the Action sheet's; the timeline's
-- `recorded_amounts` scale it.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
