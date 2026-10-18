-- Four-point Fury (Lv. 45): all nearby enemies, changes to coeurl form.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 140
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_COEURL_FORM, 0, FORM_DURATION)

    return effects
end
