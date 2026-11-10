-- Death Blossom (Lv. 38): all nearby enemies.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    ninGainNinki(effects, player, 5)
    ninBunshin(effects, player, true)

    return effects
end
