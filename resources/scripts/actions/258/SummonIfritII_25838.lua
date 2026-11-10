-- Summon Ifrit I I (Lv. 90): Ruby Ifrit's Inferno, 800 (50 % less round it); two fire attunement; Ifrit's Favor.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    smnSummon(effects, player, 1, 1, 2, 800)
    effects:gain_effect_self(SMN_EFFECT_IFRITS_FAVOR, 0, 30.0)

    return effects
end
