-- Flourish (Lv. 72): Flourishing Symmetry and Flow, Threefold Fan Dance, Fourfold Fan Dance (86) and Finishing Move Ready (96), 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DNC_EFFECT_FLOURISHING_SYMMETRY, 0, 30.0)
    effects:gain_effect_self(DNC_EFFECT_FLOURISHING_FLOW, 0, 30.0)
    effects:gain_effect_self(DNC_EFFECT_THREEFOLD_FAN_DANCE, 0, 30.0)
    if player.level >= 86 then
        effects:gain_effect_self(DNC_EFFECT_FOURFOLD_FAN_DANCE, 0, 30.0)
    end
    if player.level >= 96 then
        effects:gain_effect_self(DNC_EFFECT_FINISHING_MOVE_READY, 0, 30.0)
    end

    return effects
end
