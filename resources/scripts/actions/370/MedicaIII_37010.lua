-- Medica III (Lv. 96)
-- Heals the caster and party members within 20 yalms, and grants Regen.
-- NOTE: Plenary Indulgence's Confession bonus isn't applied yet.
POTENCY = 250
REGEN_POTENCY = 175
DURATION = 15.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:heal(whmHeal(player, POTENCY))
    effects:heal_over_time(EFFECT_MEDICA_III, DURATION, whmHeal(player, REGEN_POTENCY))

    return effects
end
