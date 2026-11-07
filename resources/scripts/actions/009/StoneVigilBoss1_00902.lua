-- Lion's Breath: the first boss's (BNpcBase 1461) short cone ahead of it (0.7 s), about every 11.5 s (1156 to the
-- player). From Bob's Chronofoil capture 1cefedf4 (2026-11-07: the Stone Vigil with Duty Support, three wipes). Its
-- area and cast are the Action sheet's; its timeline's `recorded_amounts` scales the hit to retail's.
-- Lion's Breath's hits gave status 267 for 15 s (24 of 25; EffectResult's duration).
local POTENCY = 100
local STATUS = 267
local PARAM = 0
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
