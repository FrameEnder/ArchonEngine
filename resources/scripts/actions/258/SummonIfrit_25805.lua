-- Summon Ifrit (Lv. 30): Ifrit-Egi's Inferno, 600 (50 % less round the target); two fire attunement; needs the Ruby arcanum.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    smnSummon(effects, player, 1, 1, 2, 600)
    if player.level >= 86 then
        effects:gain_effect_self(SMN_EFFECT_IFRITS_FAVOR, 0, 30.0)
    end

    return effects
end
