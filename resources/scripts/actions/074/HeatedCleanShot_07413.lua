-- Heated Clean Shot (Lv. 64): 160, 420 after Heated Slug Shot with 5 Heat and 10 Battery.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        mchShot(effects, player, 420)
        mchHeat(effects, player, 5, 10)
    else
        mchShot(effects, player, tierPotency(player, { { 94, 160 }, { 84, 120 } }))
    end

    return effects
end
