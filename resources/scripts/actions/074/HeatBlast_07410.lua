-- Heat Blast (Lv. 35): 200; an Overheated stack (the 15 s off Gauss Round and Ricochet isn't done).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 200)
    effects:consume_effect_stack(MCH_EFFECT_OVERHEATED)

    return effects
end
