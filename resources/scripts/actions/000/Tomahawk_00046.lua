-- Tomahawk (Lv. 15): ranged, more enmity (not modelled).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
