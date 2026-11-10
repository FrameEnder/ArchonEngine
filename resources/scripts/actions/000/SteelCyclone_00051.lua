-- Steel Cyclone (Lv. 45): all nearby enemies.
-- Spends 50 Beast Gauge (none under Inner Release; Global.lua's warSpendBeast; the client offers it only with 50).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 160

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    warSpendBeast(effects, player)

    return effects
end
