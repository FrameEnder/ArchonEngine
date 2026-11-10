-- Devilment (Lv. 62): 20 % more criticals and direct hits, 20 s (the rates aren't done); Flourishing Starfall (20 s) from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DNC_EFFECT_DEVILMENT, 0, 20.0)
    if player.level >= 90 then
        effects:gain_effect_self(DNC_EFFECT_FLOURISHING_STARFALL, 0, 20.0)
    end

    return effects
end
