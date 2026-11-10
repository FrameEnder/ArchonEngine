-- Improvisation (Lv. 80): 100 over 15 s to the party near while held (Rising Rhythm isn't counted).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(DNC_EFFECT_IMPROVISATION, 15.0, healAmount(player, 100))

    return effects
end
