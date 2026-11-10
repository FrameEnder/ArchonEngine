-- Hypercharge (Lv. 30): five stacks of Overheated, 10 s; 50 Heat (none with Hypercharged).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(MCH_EFFECT_HYPERCHARGED) ~= nil then
        effects:consume_effect(MCH_EFFECT_HYPERCHARGED)
    else
        effects:gauge_add("heat", -50)
    end
    effects:gain_effect_self(MCH_EFFECT_OVERHEATED, 5, 10.0)

    return effects
end
