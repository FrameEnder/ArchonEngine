-- Cure II (Lv. 30)
-- 800 after "Enhanced Healing Magic" (Lv. 85.)
BASE_POTENCY = 700
ENHANCED_POTENCY = 800

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    effects:heal(whmHeal(player, potency))

    -- Freecure makes this cast free
    if player:get_effect(EFFECT_FREECURE) ~= nil then
        effects:set_mp_cost(0)
        effects:consume_effect(EFFECT_FREECURE)
    end

    return effects
end
