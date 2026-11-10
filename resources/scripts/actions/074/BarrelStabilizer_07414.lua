-- Barrel Stabilizer (Lv. 66): Hypercharged, 30 s; Full Metal Machinist, 30 s, from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(MCH_EFFECT_HYPERCHARGED, 0, 30.0)
    if player.level >= 100 then
        effects:gain_effect_self(MCH_EFFECT_FULL_METAL_MACHINIST, 0, 30.0)
    end

    return effects
end
