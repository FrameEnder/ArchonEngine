-- Thunder II (Lv. 26), the player one: the target and enemies near it, damage over time. Uses up Thunderhead.
-- (Action 148 is the older Thunder II that NPCs use.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 60
local DOT_POTENCY = 30
local DURATION = 18.0
local EFFECT_THUNDER_II = 162
local EFFECT_THUNDERHEAD = 3870

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND)))
    effects:damage_over_time(EFFECT_THUNDER_II, DURATION, magicalDamage(player, classPotency(player, DOT_POTENCY, TRAITS_THM_MAIM_AND_MEND)))
    if player:get_effect(EFFECT_THUNDERHEAD) ~= nil then
        effects:consume_effect(EFFECT_THUNDERHEAD)
    end

    return effects
end
