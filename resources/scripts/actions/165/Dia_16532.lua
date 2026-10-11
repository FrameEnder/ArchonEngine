-- Dia (Lv. 72)
-- 85/85 after "White Magic Mastery" (Lv. 94.) NOTE: 65/65 is the pre-trait value from older patch data, unverified for 7.56.
BASE_POTENCY = 65
ENHANCED_POTENCY = 85
DURATION = 30.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_WHITE_MAGIC_MASTERY, BASE_POTENCY, ENHANCED_POTENCY)
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, potency))
    effects:damage_over_time(EFFECT_DIA, DURATION, whmDamage(player, potency))

    return effects
end
