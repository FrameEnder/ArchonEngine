-- Demolish (Lv. 30): changes to opo-opo form.
-- NOTE: the rear bonus (420) and Coeurl's Fury aren't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 360
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, FORM_DURATION)

    return effects
end
