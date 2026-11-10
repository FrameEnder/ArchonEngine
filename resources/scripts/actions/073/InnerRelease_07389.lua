-- Inner Release (Lv. 70): three stacks (15 s) making Fell Cleave and Decimate free, critical direct hits; Inner Strength (15 s); Surging Tempest 10 s longer (60 s at most); Primal Rend Ready (30 s) from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(WAR_EFFECT_INNER_RELEASE, 3, 15.0)
    effects:gain_effect_self(WAR_EFFECT_INNER_STRENGTH, 0, 15.0)
    local tempest = player:effect_time_left(WAR_EFFECT_SURGING_TEMPEST)
    if tempest ~= nil then
        effects:gain_effect_self(WAR_EFFECT_SURGING_TEMPEST, 0, math.min(tempest + 10.0, 60.0))
    end
    if player.level >= 90 then
        effects:gain_effect_self(WAR_EFFECT_PRIMAL_REND_READY, 0, 30.0)
    end

    return effects
end
