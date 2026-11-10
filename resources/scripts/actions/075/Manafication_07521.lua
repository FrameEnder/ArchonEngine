-- Manafication (Lv. 60): three stacks of Magicked Swordplay (30 s); Prefulgence Ready (30 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(RDM_EFFECT_MAGICKED_SWORDPLAY, 3, 30.0)
    if player.level >= 100 then
        effects:gain_effect_self(RDM_EFFECT_PREFULGENCE_READY, 0, 30.0)
    end

    return effects
end
