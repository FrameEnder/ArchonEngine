-- Straight Shot (Lv. 2): needs Hawk's Eye (or Barrage), which it uses up.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 200
local EFFECT_HAWKS_EYE = 3861

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, POTENCY, TRAITS_ARC_ACTION_DAMAGE)))
    if player:get_effect(EFFECT_HAWKS_EYE) ~= nil then
        effects:consume_effect(EFFECT_HAWKS_EYE)
    end

    return effects
end
