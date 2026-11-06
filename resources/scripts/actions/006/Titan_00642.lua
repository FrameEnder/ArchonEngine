-- Titan's Tumult (the Navel, BNpcBase 246): instant, round him, on everyone (the Action sheet's area): 175 to the
-- players (127-199), 163 to the Duty Support members (Bob's Chronofoil captures 650dbfd8, 0fd8bce0 and 48aaf6fe of 2026-11-06, the Navel). His timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
