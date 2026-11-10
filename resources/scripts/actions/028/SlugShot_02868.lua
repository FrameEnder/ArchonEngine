-- Slug Shot (Lv. 2): 100, 210 after Split Shot with 5 Heat.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        mchShot(effects, player, 210)
        mchHeat(effects, player, 5)
    else
        mchShot(effects, player, 100)
    end

    return effects
end
