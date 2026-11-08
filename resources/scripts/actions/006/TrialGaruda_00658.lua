-- Garuda's 658 (the Howling Eye): a blunt hit on her target and whoever else it catches (370 the median to players).
-- From Bob's Chronofoil captures of 2026-11-08 (6a5244ec, 09ecf8e7, 6c495fc4); the hit is scaled to the timeline's
-- recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
