-- Steel Cyclone (Lv. 45): all nearby enemies.
-- NOTE: the job gauge isn't implemented, so it doesn't need or spend Beast Gauge (50).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 160

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
