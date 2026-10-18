-- Twin Snakes (Lv. 18): changes to coeurl form.
-- NOTE: Raptor's Fury isn't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 420
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_COEURL_FORM, 0, FORM_DURATION)

    return effects
end
