-- Chain Stratagem (Lv. 66): the target is critically hit 10 % more, 20 s; Impact Imminent (30 s) from 92.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(SCH_EFFECT_CHAIN_STRATAGEM, 0, 20.0)
    if player.level >= 92 then
        effects:gain_effect_self(SCH_EFFECT_IMPACT_IMMINENT, 0, 30.0)
    end

    return effects
end
