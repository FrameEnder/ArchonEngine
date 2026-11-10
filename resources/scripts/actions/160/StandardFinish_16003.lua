-- Standard Finish (Lv. 15): 360 to the first enemy near, 60 % less to the rest, with no step done; the dance over.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncFinish(effects, player, DNC_EFFECT_STANDARD_STEP, 360)

    return effects
end
