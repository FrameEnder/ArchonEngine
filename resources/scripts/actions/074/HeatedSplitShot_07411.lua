-- Heated Split Shot (Lv. 54): 220; 5 Heat.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    mchShot(effects, player, tierPotency(player, { { 94, 220 }, { 84, 200 } }))
    mchHeat(effects, player, 5)

    return effects
end
