-- Temperance (Lv. 80)
-- Retail (Bob's Chronofoil captures): the White Mage gets 1872 (healing magic +20 %, 20 s; the bonus isn't implemented)
-- and the party around them 1873 (damage taken -10 %), a 5 s aura kept up while 1872 lasts; here 1873 for the 20 s.
-- NOTE: the healing bonus isn't implemented.
DURATION = 20.0
EFFECT_TEMPERANCE_PARTY = 1873
DIVINE_GRACE_DURATION = 30.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_TEMPERANCE, 0, DURATION)
    effects:gain_effect(EFFECT_TEMPERANCE_PARTY, 0, DURATION)

    -- "Enhanced Temperance" grants Divine Grace, which enables Divine Caress
    if player:has_trait(TRAIT_ENHANCED_TEMPERANCE) then
        effects:gain_effect_self(EFFECT_DIVINE_GRACE, 0, DIVINE_GRACE_DURATION)
    end

    return effects
end
