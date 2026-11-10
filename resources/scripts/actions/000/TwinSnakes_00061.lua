-- Twin Snakes (Lv. 18): changes to coeurl form.
-- NOTE: Raptor's Fury isn't implemented.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 340
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mnkUsePerfectBalance(effects, player)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect_self(STATUS_COEURL_FORM, 0, FORM_DURATION)

    return effects
end
