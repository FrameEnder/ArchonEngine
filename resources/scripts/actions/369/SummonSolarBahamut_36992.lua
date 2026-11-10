-- Summon Solar Bahamut (Lv. 100): Solar Bahamut and Lightwyrm Trance (15 s, as Dreadwyrm here); Refulgent Lux (30 s); all three arcana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SMN_EFFECT_DREADWYRM_TRANCE, 0, 15.0)
    effects:gauge_set("arcana", SMN_ARCANA_ALL)
    effects:gain_effect_self(SMN_EFFECT_REFULGENT_LUX, 0, 30.0)

    return effects
end
