-- Rainbow Drip (Lv. 92): 1,000 to the first enemy in a line, 85 % less to the rest; a White Paint; Rainbow Bright spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMagic(effects, player, 1000, 85)
    pctWhitePaint(effects, player)
    if player:get_effect(PCT_EFFECT_RAINBOW_BRIGHT) ~= nil then
        effects:consume_effect(PCT_EFFECT_RAINBOW_BRIGHT)
    end

    return effects
end
