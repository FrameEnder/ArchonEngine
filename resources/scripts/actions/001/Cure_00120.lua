-- Cure (Lv. 2)
-- 500 after "Enhanced Healing Magic" (Lv. 85.)
BASE_POTENCY = 450
ENHANCED_POTENCY = 500
FREECURE_CHANCE = 0.15
FREECURE_DURATION = 15.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    effects:heal(whmHeal(player, potency))

    -- "Freecure" trait: 15% chance the next Cure II costs no MP
    if player:has_trait(TRAIT_FREECURE) and math.random() < FREECURE_CHANCE then
        effects:gain_effect_self(EFFECT_FREECURE, 0, FREECURE_DURATION)
    end

    return effects
end
