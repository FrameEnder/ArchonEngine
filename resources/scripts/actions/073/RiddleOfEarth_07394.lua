-- Riddle Of Earth (Lv. 64): 20 % less damage for 10 s; Earth's Rumination (30 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MNK_EFFECT_RIDDLE_OF_EARTH, 0, 10.0)
    effects:gain_effect_self(MNK_EFFECT_EARTHS_RUMINATION, 0, 30.0)

    return effects
end
