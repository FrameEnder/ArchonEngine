-- Overpower (Lv. 10): all nearby enemies, more enmity (not modelled).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 110

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
