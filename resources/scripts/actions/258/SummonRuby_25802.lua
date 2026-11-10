-- Summon Ruby (Lv. 6): Ruby Carbuncle's Glittering Ruby, 400; two fire attunement (30 s); needs the Ruby arcanum.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    smnSummon(effects, player, 1, 1, 2, 400)

    return effects
end
