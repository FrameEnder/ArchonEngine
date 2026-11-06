-- Titan's Earthen Fury (the Navel, BNpcBase 246): instant, from the arena's middle as the Heart of the Mountain falls,
-- on everyone: 427-719 to the players (630), 258-634 to the Duty Support members (Bob's Chronofoil captures 650dbfd8, 0fd8bce0 and 48aaf6fe of 2026-11-06, the Navel, the Heart broken in all
-- three). His timeline's `recorded_amounts` scale it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
