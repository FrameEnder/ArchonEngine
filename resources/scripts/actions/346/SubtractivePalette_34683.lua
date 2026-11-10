-- Subtractive Palette (Lv. 60): three stacks of Subtractive Palette, 30 s; Monochrome Tones from 90; 50 Palette (none with Subtractive Spectrum).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(PCT_EFFECT_SUBTRACTIVE_SPECTRUM) ~= nil then
        effects:consume_effect(PCT_EFFECT_SUBTRACTIVE_SPECTRUM)
    else
        effects:gauge_add("palette", -50)
    end
    effects:gain_effect_self(PCT_EFFECT_SUBTRACTIVE_PALETTE, 3, 30.0)
    if player.level >= 90 then
        effects:gain_effect_self(PCT_EFFECT_MONOCHROME_TONES, 0, 30.0)
    end

    return effects
end
