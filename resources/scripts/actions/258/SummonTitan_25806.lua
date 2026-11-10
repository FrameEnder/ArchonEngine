-- Summon Titan (Lv. 35): Titan-Egi's Earthen Fury, 600 (50 % less round it); four earth attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    smnSummon(effects, player, 2, 2, 4, 600)

    return effects
end
