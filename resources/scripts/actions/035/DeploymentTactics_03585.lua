-- Deployment Tactics (Lv. 56): spreads the target's Galvanize to the party near (here: Galvanize on the target, 30 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SCH_EFFECT_GALVANIZE, 0, 30.0)

    return effects
end
