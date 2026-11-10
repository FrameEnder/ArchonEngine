-- Dreadwyrm Trance (Lv. 58): Dreadwyrm Trance (15 s): Astral Impulse and Astral Flare; all three arcana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(SMN_EFFECT_DREADWYRM_TRANCE, 0, 15.0)
    effects:gauge_set("arcana", SMN_ARCANA_ALL)

    return effects
end
