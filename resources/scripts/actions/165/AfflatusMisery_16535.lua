-- Afflatus Misery (Lv. 74)
-- Hits the target and enemies within 5 yalms of it. Everyone after the first takes 50% less damage.
-- Requires the Blood Lily to be in full bloom (the client won't let you use it otherwise.)
-- 1,400 after "White Magic Mastery" (Lv. 94.) NOTE: 1,240 is the pre-trait value from older patch data, unverified for 7.56.
BASE_POTENCY = 1240
ENHANCED_POTENCY = 1400

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_WHITE_MAGIC_MASTERY, BASE_POTENCY, ENHANCED_POTENCY)
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, potency))
    effects:aoe_falloff(50)
    effects:spend_blood_lily()

    return effects
end
