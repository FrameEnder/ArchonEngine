-- Succor (Lv. 35): heals self and nearby party members, with Galvanize (a barrier), 30 s.
-- NOTE: barriers don't absorb damage yet; the status is applied.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 200
local EFFECT_GALVANIZE = 297
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    -- Recitation (74): free and surely critical.
    if player:get_effect(SCH_EFFECT_RECITATION) ~= nil then
        effects:guarantee_critical()
        effects:consume_effect(SCH_EFFECT_RECITATION)
    end
    effects:heal(healAmount(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gain_effect(EFFECT_GALVANIZE, 0, DURATION)

    return effects
end
