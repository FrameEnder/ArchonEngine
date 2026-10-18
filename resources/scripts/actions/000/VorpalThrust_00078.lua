-- Vorpal Thrust (Lv. 4), after True Thrust.
-- Also used by Miraudont in a quest battle (recorded_amounts there); NPCs never have a combo, so they keep NPC_POTENCY.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 140
local POTENCY = 130
local COMBO_POTENCY = 280

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, NPC_POTENCY))
        return effects
    end

    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, potency))

    return effects
end
