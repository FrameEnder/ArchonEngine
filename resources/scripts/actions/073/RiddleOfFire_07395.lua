-- Riddle Of Fire (Lv. 68): 15 % more damage for 20 s; Fire's Rumination (20 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MNK_EFFECT_RIDDLE_OF_FIRE, 0, 20.0)
    if player.level >= 100 then
        effects:gain_effect_self(MNK_EFFECT_FIRES_RUMINATION, 0, 20.0)
    end

    return effects
end
