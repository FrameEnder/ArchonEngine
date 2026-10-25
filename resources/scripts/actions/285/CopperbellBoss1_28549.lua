-- The Copperbell Mines' first boss (BNpcBase 14460): instant, as it lands on the first and second of its three leaps
-- (Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25.) Nobody was caught by one in the three runs, so what
-- it does is a stand-in: its timeline's `recorded_amounts` says how hard. Its area is the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
