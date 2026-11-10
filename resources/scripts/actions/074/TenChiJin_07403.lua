-- Ten Chi Jin (Lv. 70): the three mudras as Ninjutsu for 6 s (here: Ten Chi Jin, then three Ninjutsu by mudra); Tenri Jindo Ready (30 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(NIN_EFFECT_TEN_CHI_JIN, 3, 6.0)
    if player.level >= 100 then
        effects:gain_effect_self(NIN_EFFECT_TENRI_JINDO_READY, 0, 30.0)
    end

    return effects
end
