-- Heavy Shot (Lv. 1): 20% chance of Hawk's Eye (Heavier Shot, Lv. 2).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 160
local POTENCY = 160
local EFFECT_HAWKS_EYE = 3861
local HAWKS_EYE_CHANCE = 0.2
local HAWKS_EYE_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, NPC_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, POTENCY, TRAITS_ARC_ACTION_DAMAGE)))
    if player:has_trait(17) and math.random() < HAWKS_EYE_CHANCE then
        effects:gain_effect_self(EFFECT_HAWKS_EYE, 0, HAWKS_EYE_DURATION)
    end
    brdRepertoire(effects, player)

    return effects
end
