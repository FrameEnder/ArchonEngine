-- Blazing Shot (Lv. 68): 240; an Overheated stack (the 15 s off Double Check and Checkmate isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 240)
    effects:consume_effect_stack(MCH_EFFECT_OVERHEATED)

    return effects
end
