-- Regen (Lv. 35)
-- 250 after "Enhanced Healing Magic" (Lv. 85.)
BASE_POTENCY = 200
ENHANCED_POTENCY = 250
DURATION = 18.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    effects:heal_over_time(EFFECT_REGEN, DURATION, whmHeal(player, potency))

    return effects
end
