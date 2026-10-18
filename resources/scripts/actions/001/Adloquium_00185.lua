-- Adloquium (Lv. 30): heals and grants Galvanize (a barrier), 30 s.
-- NOTE: barriers don't absorb damage yet; the status is applied.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 300
local EFFECT_GALVANIZE = 297
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_effect(EFFECT_GALVANIZE, 0, DURATION)

    return effects
end
