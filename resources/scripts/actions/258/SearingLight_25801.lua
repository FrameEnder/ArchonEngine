-- Searing Light (Lv. 66): 5 % more damage for the party near, 20 s; Ruby's Glimmer (30 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SMN_EFFECT_SEARING_LIGHT, 0, 20.0)
    if player.level >= 96 then
        effects:gain_effect_self(SMN_EFFECT_RUBYS_GLIMMER, 0, 30.0)
    end

    return effects
end
