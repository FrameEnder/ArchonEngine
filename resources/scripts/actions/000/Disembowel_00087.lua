-- Disembowel (Lv. 18), after True Thrust. Combo bonus: Power Surge (+10% damage, 30 s).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 140
local COMBO_POTENCY = 250
local EFFECT_POWER_SURGE = 2720
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, potency))
    if in_combo then
        effects:gain_effect_self(EFFECT_POWER_SURGE, 0, DURATION)
    end

    return effects
end
