-- Maim (Lv. 4), after Heavy Swing.
-- NOTE: the job gauge isn't implemented, so the combo's Beast Gauge (+10) isn't given.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 190
local COMBO_POTENCY = 340

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
