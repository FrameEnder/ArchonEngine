-- Snap Punch (Lv. 6): changes to opo-opo form.
-- NOTE: the flank bonus (330) and Coeurl's Fury aren't implemented.
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 230
local POTENCY = 270
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, FORM_DURATION)
        return effects
    end

    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, FORM_DURATION)

    return effects
end
