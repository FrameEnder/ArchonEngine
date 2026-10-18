-- Vorpal Thrust (Lv. 4), after True Thrust.
-- Also used by Miraudont in a quest battle (recorded_amounts there); NPCs never have a combo, so they keep NPC_POTENCY.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 140
local POTENCY = 130
local COMBO_POTENCY = 240

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
