-- Cure III (Lv. 40)
-- Heals the target and party members within 10 yalms of them.
-- 600 after "Enhanced Healing Magic" (Lv. 85.)
-- NOTE: Plenary Indulgence's Confession bonus isn't applied yet.
BASE_POTENCY = 550
ENHANCED_POTENCY = 600

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    effects:heal(whmHeal(player, potency))

    return effects
end
