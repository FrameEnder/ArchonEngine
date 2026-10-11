-- Glare III (Lv. 82)
-- 350 after "White Magic Mastery" (Lv. 94.) NOTE: 310 is the pre-trait value from older patch data, unverified for 7.56.
BASE_POTENCY = 310
ENHANCED_POTENCY = 350

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_WHITE_MAGIC_MASTERY, BASE_POTENCY, ENHANCED_POTENCY)
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, potency))

    return effects
end
