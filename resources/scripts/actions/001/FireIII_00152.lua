-- Fire III (Lv. 35). With Firestarter (which it uses up) it costs no MP.
-- NOTE: Firestarter's instant cast isn't implemented (the cast time is the engine's).
-- NOTE: Astral Fire and Umbral Ice (the job gauge) aren't implemented: no potency or MP changes from them. Since 7.x Thunder spells need Thunderhead; without the gauge, changing element (this) grants it.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 290
local EFFECT_FIRESTARTER = 165
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND)))
    effects:gain_effect_self(EFFECT_THUNDERHEAD, 0, THUNDERHEAD_DURATION)
    if player:get_effect(EFFECT_FIRESTARTER) ~= nil then
        effects:set_mp_cost(0)
        effects:consume_effect(EFFECT_FIRESTARTER)
    end

    return effects
end
