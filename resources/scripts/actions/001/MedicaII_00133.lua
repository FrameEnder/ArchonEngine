-- Medica II (Lv. 50)
-- Heals the caster and party members within 20 yalms, and grants Regen.
-- 250 + 150 Regen after "Enhanced Healing Magic" (Lv. 85.)
BASE_POTENCY = 200
BASE_REGEN_POTENCY = 100
ENHANCED_POTENCY = 250
ENHANCED_REGEN_POTENCY = 150
DURATION = 15.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    local potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_POTENCY, ENHANCED_POTENCY)
    local regen_potency = traitPotency(player, TRAIT_ENHANCED_HEALING_MAGIC, BASE_REGEN_POTENCY, ENHANCED_REGEN_POTENCY)
    effects:heal(whmHeal(player, potency))
    effects:heal_over_time(EFFECT_MEDICA_II, DURATION, whmHeal(player, regen_potency))

    return effects
end
