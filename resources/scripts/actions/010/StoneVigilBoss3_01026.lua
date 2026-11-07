-- Cauterize: the last boss's (BNpcBase 1455) dive down a wide line across the room (3.7 s), as the director says (1070
-- to the player). From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty Support, three wipes).
-- Its area and cast are the Action sheet's; its timeline's `recorded_amounts` scales the hit to retail's.
-- Its hits gave status 268 for 15 s.
local POTENCY = 100
local STATUS = 268
local PARAM = 0
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
