-- Summon Titan I I (Lv. 90): Topaz Titan's Earthen Fury, 800 (50 % less round it); four earth attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    smnSummon(effects, player, 2, 2, 4, 800)

    return effects
end
