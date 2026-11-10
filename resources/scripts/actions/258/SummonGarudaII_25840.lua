-- Summon Garuda I I (Lv. 90): Emerald Garuda's Aerial Blast, 800 (50 % less round it); four wind attunement; Garuda's Favor.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    smnSummon(effects, player, 4, 3, 4, 800)
    effects:gain_effect_self(SMN_EFFECT_GARUDAS_FAVOR, 0, 30.0)

    return effects
end
