-- Unveiled Gibbet (Lv. 70): 440; 50 Soul; Soul Reaver.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rprDamage(effects, player, 440)
    effects:gauge_add("soul", -50)
    if player.level >= 70 then
        effects:gain_effect_self(RPR_EFFECT_SOUL_REAVER, 1, 30.0)
    end

    return effects
end
