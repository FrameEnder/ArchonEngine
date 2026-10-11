-- Medica (Lv. 10)
-- Heals the caster and party members within 20 yalms.
-- 400 after "Enhanced Healing Magic" (Lv. 85.)
-- NOTE: Plenary Indulgence's Confession bonus isn't applied yet.
BASE_POTENCY = 300
ENHANCED_POTENCY = 400

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    effects:heal(whmHeal(player, potency))

    return effects
end
