-- Reassemble (Lv. 10): the next weaponskill a sure critical direct hit, 5 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MCH_EFFECT_REASSEMBLED, 0, 5.0)

    return effects
end
