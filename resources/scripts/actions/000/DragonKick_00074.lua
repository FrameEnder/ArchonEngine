-- Dragon Kick (Lv. 50): changes to raptor form.
-- NOTE: Opo-opo's Fury isn't implemented.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 250
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_RAPTOR_FORM, 0, FORM_DURATION)

    return effects
end
