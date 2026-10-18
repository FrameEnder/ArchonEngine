-- Wide Volley (Lv. 25), the player one: the target and enemies near it; needs Hawk's Eye or Barrage, which it uses up.
-- (Action 111 is the older Wide Volley that NPCs use.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 140
local BARRAGE_POTENCY = 220
local EFFECT_HAWKS_EYE = 3861
local EFFECT_BARRAGE = 128

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if player:get_effect(EFFECT_BARRAGE) ~= nil then
        potency = BARRAGE_POTENCY
        effects:consume_effect(EFFECT_BARRAGE)
    else
        effects:consume_effect(EFFECT_HAWKS_EYE)
    end
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, potency, TRAITS_ARC_ACTION_DAMAGE)))

    return effects
end
