-- The Praetorium's launchers' 29049 (BNpcBase 6358, unseen: `launchers.json`): a 2.7 s cast round the launcher itself,
-- 600 to everyone it caught, players and Duty Support members alike, every time (39 hits in Connor's Chronofoil capture
-- af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18, five runs), of no damage type
-- or element (its byte 0.) Its area is the Action sheet's; the timeline's `recorded_amounts` (Launcher_6358.json) make
-- it the 600.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_UNIQUE, magicalDamage(player, POTENCY))

    return effects
end
