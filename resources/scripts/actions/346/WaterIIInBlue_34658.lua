-- Water I I In Blue (Lv. 45): 220 to the target and round it with Aetherhues II; 25 Palette (60) and a White Paint (80).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 220 }, { 74, 190 }, { 1, 160 } }))
    pctHue(effects, player, PCT_EFFECT_AETHERHUES_II, nil)
    if player.level >= 60 then
        effects:gauge_add("palette", 25)
    end
    pctWhitePaint(effects, player)

    return effects
end
