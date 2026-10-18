-- Rockbreaker (Lv. 30): all nearby enemies, changes to opo-opo form.
-- Also used by Yda in a quest battle, where what it does comes from the recording (recorded_amounts).
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local NPC_POTENCY = 130
local POTENCY = 150
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        return effects
    end

    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, FORM_DURATION)

    return effects
end
