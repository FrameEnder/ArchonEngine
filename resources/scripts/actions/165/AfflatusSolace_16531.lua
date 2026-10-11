-- Afflatus Solace (Lv. 52)
-- Costs a Lily (the client won't let you use it without one), and nourishes the Blood Lily.
-- 800 after "Enhanced Healing Magic" (Lv. 85.)
BASE_POTENCY = 700
ENHANCED_POTENCY = 800

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    effects:heal(whmHeal(player, potency))
    effects:spend_lily()

    return effects
end
