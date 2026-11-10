-- Standard Step (Lv. 15): the dance (15 s): two steps.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncStartDance(effects, DNC_EFFECT_STANDARD_STEP, 2)

    return effects
end
