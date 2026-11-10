-- Hunters Bite (Lv. 40): 180 to all enemies near; Hunter's Instinct.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 180)
    effects:gain_effect_self(VPR_EFFECT_HUNTERS_INSTINCT, 0, 40.0)

    return effects
end
