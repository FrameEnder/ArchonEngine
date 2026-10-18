-- Piercing Talon (Lv. 15): ranged; stronger with Enhanced Piercing Talon (from Elusive Jump), which it uses up.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 200
local ENHANCED_POTENCY = 350
local EFFECT_ENHANCED_PIERCING_TALON = 1870

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if player:get_effect(EFFECT_ENHANCED_PIERCING_TALON) ~= nil then
        potency = ENHANCED_POTENCY
        effects:consume_effect(EFFECT_ENHANCED_PIERCING_TALON)
    end
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, potency))

    return effects
end
