-- Enshroud (Lv. 80): Enshrouded and five Lemure Shroud, 30 s; 50 Shroud (none with Ideal Host); Oblatio (30 s) from 92.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(RPR_EFFECT_IDEAL_HOST) ~= nil then
        effects:consume_effect(RPR_EFFECT_IDEAL_HOST)
    else
        effects:gauge_add("shroud", -50)
    end
    effects:gain_effect_self(RPR_EFFECT_ENSHROUDED, 0, 30.0)
    effects:gauge_set("lemure", 5)
    effects:gauge_set("void", 0)
    if player.level >= 92 then
        effects:gain_effect_self(RPR_EFFECT_OBLATIO, 0, 30.0)
    end

    return effects
end
