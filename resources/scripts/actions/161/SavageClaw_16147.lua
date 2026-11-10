-- Savage Claw (Lv. 60): 500 after Gnashing Fang; Ready to Tear (10 s) from 70.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, tierPotency(player, { { 94, 500 }, { 1, 460 } }))
    if player.level >= 70 then
        effects:gain_effect_self(GNB_EFFECT_READY_TO_TEAR, 0, 10.0)
    end

    return effects
end
