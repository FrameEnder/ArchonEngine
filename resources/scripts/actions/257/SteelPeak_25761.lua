-- Steel Peak (Lv. 15).
-- NOTE: the job gauge isn't implemented, so it doesn't need or spend the five chakra.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 180

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
