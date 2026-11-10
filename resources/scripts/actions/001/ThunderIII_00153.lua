-- Thunder III (Lv. 45, replaces Thunder): lightning damage over time. Uses up Thunderhead.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 120
local DOT_POTENCY = 50
local DURATION = 27.0
local EFFECT_THUNDER_III = 163
local EFFECT_THUNDERHEAD = 3870

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, math.floor(blmEnochian(player) * classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    effects:damage_over_time(EFFECT_THUNDER_III, DURATION, magicalDamage(player, math.floor(blmEnochian(player) * classPotency(player, DOT_POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    if player:get_effect(EFFECT_THUNDERHEAD) ~= nil then
        effects:consume_effect(EFFECT_THUNDERHEAD)
    end

    return effects
end
