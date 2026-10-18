-- Thunder (Lv. 6): lightning damage over time. Uses up Thunderhead.
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster. (They keep their damage over time.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 100
local NPC_DOT_POTENCY = 45
local POTENCY = 100
local DOT_POTENCY = 45
local DURATION = 24.0
local EFFECT_THUNDER = 161
local EFFECT_THUNDERHEAD = 3870

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, NPC_POTENCY))
        effects:damage_over_time(EFFECT_THUNDER, DURATION, magicalDamage(player, NPC_DOT_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND)))
    effects:damage_over_time(EFFECT_THUNDER, DURATION, magicalDamage(player, classPotency(player, DOT_POTENCY, TRAITS_THM_MAIM_AND_MEND)))
    if player:get_effect(EFFECT_THUNDERHEAD) ~= nil then
        effects:consume_effect(EFFECT_THUNDERHEAD)
    end

    return effects
end
