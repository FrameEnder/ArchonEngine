-- Heated Slug Shot (Lv. 60): 140, 320 after Heated Split Shot with 5 Heat.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        mchShot(effects, player, 320)
        mchHeat(effects, player, 5)
    else
        mchShot(effects, player, tierPotency(player, { { 94, 140 }, { 84, 120 } }))
    end

    return effects
end
