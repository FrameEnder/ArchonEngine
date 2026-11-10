-- Arcane Circle (Lv. 72): 3 % more damage for the party near, 20 s; Immortal Sacrifice from the party's actions in its 5 s (here: four stacks, a light party's).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(RPR_EFFECT_ARCANE_CIRCLE, 0, 20.0)
    if player.level >= 88 then
        effects:gain_effect_self(RPR_EFFECT_IMMORTAL_SACRIFICE, 4, 30.0)
        effects:gain_effect_self(RPR_EFFECT_BLOODSOWN_CIRCLE, 0, 6.0)
    end

    return effects
end
