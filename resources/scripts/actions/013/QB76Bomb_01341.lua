-- Heart of the Forest's third wave's 1341 (quest battle 76, BNpcBase 1704): instant, on the player as the one that
-- walked up to them reaches them: 448, 672, 672 and 897 in four uses (Bob's Chronofoil capture 667a68c5 of
-- 2026-11-16; multiples of 224, by what isn't known), the timeline's recorded amount; it dies with it (the director,
-- content/questbattle/76.lua.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
