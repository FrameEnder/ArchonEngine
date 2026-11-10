-- Clean Shot (Lv. 26): 100, 270 after Slug Shot with 5 Heat and 10 Battery.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        mchShot(effects, player, 270)
        mchHeat(effects, player, 5, 10)
    else
        mchShot(effects, player, 100)
    end

    return effects
end
