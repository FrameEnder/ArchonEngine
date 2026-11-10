-- Technical Step (Lv. 70): the dance (15 s): four steps.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncStartDance(effects, DNC_EFFECT_TECHNICAL_STEP, 4)

    return effects
end
