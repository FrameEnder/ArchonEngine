-- Garuda's 656 (the Howling Eye): a magic hit on her target and whoever else it catches (444 the median to players, 408
-- to Duty Support's members, 942 to a monolith). From Bob's Chronofoil captures of 2026-11-08 (6a5244ec, 09ecf8e7,
-- 6c495fc4); the hit is scaled to the timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
