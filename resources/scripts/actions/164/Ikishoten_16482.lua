-- Ikishoten (Lv. 68): 50 Kenki; Ogi Namikiri Ready (30 s) from 90, Zanshin Ready (30 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("kenki", 50)
    if player.level >= 90 then
        effects:gain_effect_self(SAM_EFFECT_OGI_NAMIKIRI_READY, 0, 30.0)
    end
    if player.level >= 96 then
        effects:gain_effect_self(SAM_EFFECT_ZANSHIN_READY, 0, 30.0)
    end

    return effects
end
