-- Ninjutsu (Lv. 30): the ninjutsu for the mudras made before it.
-- NOTE: mudras and Ninjutsu combinations (and the Ninki gauge) aren't implemented: it always does Fuma Shuriken (potency a guess) and uses up Mudra and Kassatsu.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 450
local EFFECT_MUDRA = 496
local EFFECT_KASSATSU = 497

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    if player:get_effect(EFFECT_MUDRA) ~= nil then
        effects:consume_effect(EFFECT_MUDRA)
    end
    if player:get_effect(EFFECT_KASSATSU) ~= nil then
        effects:consume_effect(EFFECT_KASSATSU)
    end

    return effects
end
