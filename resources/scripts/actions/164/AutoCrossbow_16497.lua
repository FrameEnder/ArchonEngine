-- Auto Crossbow (Lv. 52): 180 to the enemies in a cone; an Overheated stack.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, 180, true)
    effects:consume_effect_stack(MCH_EFFECT_OVERHEATED)

    return effects
end
