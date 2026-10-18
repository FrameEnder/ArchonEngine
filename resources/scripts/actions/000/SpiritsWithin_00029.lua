-- Spirits Within (Lv. 30). Restores MP.
-- Also used by the handsome stranger in Underneath the Sultantree's quest battle (recorded_amounts there.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 270
local MP = 500

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    effects:restore_mp(MP)

    return effects
end
