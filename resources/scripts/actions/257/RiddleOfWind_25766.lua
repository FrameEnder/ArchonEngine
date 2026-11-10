-- Riddle Of Wind (Lv. 72): auto-attacks twice as fast for 15 s; Wind's Rumination (15 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MNK_EFFECT_RIDDLE_OF_WIND, 0, 15.0)
    if player.level >= 96 then
        effects:gain_effect_self(MNK_EFFECT_WINDS_RUMINATION, 0, 15.0)
    end

    return effects
end
