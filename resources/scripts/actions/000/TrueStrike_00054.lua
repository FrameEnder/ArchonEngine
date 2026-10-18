-- True Strike (Lv. 4): changes to coeurl form.
-- NOTE: Raptor's Fury isn't implemented.
-- NPCs use this ID too (quest battles): they keep NPC_POTENCY and nothing else, see isPlayerCaster.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local NPC_POTENCY = 260
local POTENCY = 270
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not isPlayerCaster(player) then
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, NPC_POTENCY))
        effects:gain_effect_self(STATUS_COEURL_FORM, 0, FORM_DURATION)
        return effects
    end

    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_COEURL_FORM, 0, FORM_DURATION)

    return effects
end
