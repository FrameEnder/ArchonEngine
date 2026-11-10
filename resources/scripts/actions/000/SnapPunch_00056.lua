-- Snap Punch (Lv. 6): changes to opo-opo form.
-- NOTE: the flank bonus (330) and Coeurl's Fury aren't implemented.
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 230
local POTENCY = 240
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mnkUsePerfectBalance(effects, player)
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, FORM_DURATION)
        return effects
    end

    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, FORM_DURATION)

    return effects
end
