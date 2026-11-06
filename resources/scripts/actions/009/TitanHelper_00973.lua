-- Titan's helper's Weight of the Land (the Navel, BNpcBase 434): a 3.2 s cast where the director put it, under one of
-- the party, round it (the Action sheet's area): 476-491 to the player, 442-458 to the Duty Support members (Bob's Chronofoil captures 650dbfd8, 0fd8bce0 and 48aaf6fe of 2026-11-06, the Navel).
-- Its timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
