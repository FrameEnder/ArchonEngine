-- Succor (Lv. 35): heals self and nearby party members, with Galvanize (a barrier), 30 s.
-- Galvanize absorbs 160 % of the HP restored (the 7.x tooltip; `server/barrier.rs`, retail's share of it in EffectResult
-- isn't pinned to a ratio yet: Bob's captures gave 2.43-2.50 times the heal for nine Adloquiums, other numbers for four.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 200
local EFFECT_GALVANIZE = 297
local DURATION = 30.0
local BARRIER_PERCENT = 160

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    -- Recitation (74): free and surely critical.
    if player:get_effect(SCH_EFFECT_RECITATION) ~= nil then
        effects:guarantee_critical()
        effects:consume_effect(SCH_EFFECT_RECITATION)
    end
    local healed = healAmount(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND))
    effects:heal(healed)
    effects:gain_effect(EFFECT_GALVANIZE, 0, DURATION)
    effects:barrier(EFFECT_GALVANIZE, math.floor(healed * BARRIER_PERCENT / 100))

    return effects
end
