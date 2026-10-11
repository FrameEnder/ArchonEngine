-- Temperance (Lv. 80)
-- NOTE: The status is applied, but the healing bonus and damage reduction aren't implemented yet.
DURATION = 20.0
DIVINE_GRACE_DURATION = 30.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_TEMPERANCE, 0, DURATION)

    -- "Enhanced Temperance" grants Divine Grace, which enables Divine Caress
    if player:has_trait(TRAIT_ENHANCED_TEMPERANCE) then
        effects:gain_effect_self(EFFECT_DIVINE_GRACE, 0, DIVINE_GRACE_DURATION)
    end

    return effects
end
