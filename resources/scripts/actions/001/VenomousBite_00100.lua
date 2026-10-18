-- Venomous Bite (Lv. 6): damage over time.
-- Also used by the Gods' Quiver bows in a quest battle (recorded_amounts there).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 100
local POTENCY = 100
local DOT_POTENCY = 15
local DURATION = 45.0
local EFFECT_VENOMOUS_BITE = 124

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, NPC_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, POTENCY, TRAITS_ARC_ACTION_DAMAGE)))
    effects:damage_over_time(EFFECT_VENOMOUS_BITE, DURATION, physicalDamage(player, classPotency(player, DOT_POTENCY, TRAITS_ARC_ACTION_DAMAGE)))

    return effects
end
