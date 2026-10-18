-- Trick Attack (Lv. 18): the target takes 10% more damage from you for 15 s. Needs Hidden, which it uses up.
-- NOTE: the rear bonus (400) isn't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 300
local EFFECT_TRICK_ATTACK = 3254
local DURATION = 15.0
local EFFECT_HIDDEN = 614

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    effects:gain_effect(EFFECT_TRICK_ATTACK, 0, DURATION)
    if player:get_effect(EFFECT_HIDDEN) ~= nil then
        effects:consume_effect(EFFECT_HIDDEN)
    end

    return effects
end
