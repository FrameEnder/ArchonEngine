-- Howling Fist (Lv. 40): enemies in a line.
-- Spends the five chakra (the job gauge's "chakra"; the client offers it only with five).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gauge_add("chakra", -5)

    return effects
end
