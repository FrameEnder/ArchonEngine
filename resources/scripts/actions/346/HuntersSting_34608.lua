-- Hunters Sting (Lv. 5): 300 after Steel or Reaving Fangs; Hunter's Instinct (10 % more damage, 40 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 300)
    effects:gain_effect_self(VPR_EFFECT_HUNTERS_INSTINCT, 0, 40.0)

    return effects
end
