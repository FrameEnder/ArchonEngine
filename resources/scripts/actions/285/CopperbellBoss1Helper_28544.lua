-- The Copperbell Mines' first boss's helpers (BNpcBase 9020, layout 9054923): the 8.7 s marker on the spot Kottos leaps
-- onto next (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25: three a round, 2 s apart). Nobody was caught
-- by one in the three runs, so what it does is a stand-in: its timeline's `recorded_amounts` (CopperbellBoss1Helper_9020)
-- says how hard. Its area is the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
