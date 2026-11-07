-- An action of the Stone Vigil's 1464s, about every 34 s. From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone
-- Vigil with Duty Support, three wipes). Its area and cast are the Action sheet's; its timeline's `recorded_amounts`
-- scales the hit to retail's.
-- Its hits gave status 18 (param 50) for 30 s.
local POTENCY = 100
local STATUS = 18
local PARAM = 50
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
