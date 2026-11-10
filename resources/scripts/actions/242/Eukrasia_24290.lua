-- Eukrasia (Lv. 30): Eukrasia: the next Dosis, Diagnosis, Prognosis or Dyskrasia is its Eukrasian one.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_set("eukrasia", 1)
    effects:gain_effect_self(SGE_EFFECT_EUKRASIA, 0, 0.0)

    return effects
end
