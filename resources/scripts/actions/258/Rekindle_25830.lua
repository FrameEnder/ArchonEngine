-- Rekindle (Lv. 80): heals 400 and 200 over 15 s; only in Firebird Trance.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, 400))
    effects:gain_effect(SMN_EFFECT_REKINDLE, 0, 15.0)

    return effects
end
