-- Water In Blue (Lv. 15): 570 with Aetherhues II; 25 Palette (60) and a White Paint (80).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, tierPotency(player, { { 94, 570 }, { 84, 500 }, { 74, 460 }, { 54, 390 }, { 1, 360 } }))
    pctHue(effects, player, PCT_EFFECT_AETHERHUES_II, nil)
    if player.level >= 60 then
        effects:gauge_add("palette", 25)
    end
    pctWhitePaint(effects, player)

    return effects
end
