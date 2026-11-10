-- Fire III (Lv. 35). With Firestarter (which it uses up) it costs no MP.
-- NOTE: Firestarter's instant cast isn't implemented (the cast time is the engine's).
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 290
local MP_COST = 2000
local EFFECT_FIRESTARTER = 165
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 0.0 -- no time limit on retail (Bob's Chronofoil captures; the Status sheet's permanent)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmFirePotency(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    effects:set_mp_cost(blmFireCost(player, MP_COST))
    -- Astral Fire III, Umbral Ice gone.
    blmSetElement(effects, player, 3, false)
    if player:get_effect(EFFECT_FIRESTARTER) ~= nil then
        effects:set_mp_cost(0)
        effects:consume_effect(EFFECT_FIRESTARTER)
    end

    return effects
end
