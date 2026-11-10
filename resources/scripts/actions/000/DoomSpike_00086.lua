-- Doom Spike (Lv. 40): enemies in a line.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 110

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, POTENCY))

    return effects
end
