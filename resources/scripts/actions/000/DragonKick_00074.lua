-- Dragon Kick (Lv. 50): changes to raptor form.
-- NOTE: Opo-opo's Fury isn't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 320
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_RAPTOR_FORM, 0, FORM_DURATION)

    return effects
end
