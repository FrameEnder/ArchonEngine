-- Windbite (Lv. 30): wind damage over time.
-- Also used by Galfrid in a quest battle (recorded_amounts there).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 100
local POTENCY = 60
local DOT_POTENCY = 20
local DURATION = 45.0
local EFFECT_WINDBITE = 129

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, NPC_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, POTENCY, TRAITS_ARC_ACTION_DAMAGE)))
    effects:damage_over_time(EFFECT_WINDBITE, DURATION, physicalDamage(player, classPotency(player, DOT_POTENCY, TRAITS_ARC_ACTION_DAMAGE)))

    if isPlayerCaster(player) then
        -- 35 % Hawk's Eye from Bite Mastery II (76).
        if player.level >= 76 then
            brdHawksEye(effects, 0.35)
        end
        brdRepertoire(effects, player)
    end

    return effects
end
