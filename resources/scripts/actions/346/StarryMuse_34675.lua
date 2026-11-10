-- Starry Muse (Lv. 70): 5 % more damage for the party near, 20 s; Subtractive Spectrum (30 s), five Hyperphantasia (82, 30 s), Starstruck (100, 20 s); the starry sky off the canvas.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(PCT_EFFECT_STARRY_MUSE, 0, 20.0)
    effects:gain_effect_self(PCT_EFFECT_SUBTRACTIVE_SPECTRUM, 0, 30.0)
    if player.level >= 82 then
        effects:gain_effect_self(PCT_EFFECT_HYPERPHANTASIA, 5, 30.0)
    end
    if player.level >= 100 then
        effects:gain_effect_self(PCT_EFFECT_STARSTRUCK, 0, 20.0)
    end
    effects:gauge_set("canvas", player:gauge("canvas") & 31)

    return effects
end
