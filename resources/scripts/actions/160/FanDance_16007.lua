-- Fan Dance (Lv. 30): 180; a Feather; Threefold Fan Dance half the time (30 s) from 66.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, tierPotency(player, { { 94, 180 }, { 1, 150 } }))
    effects:gauge_add("feathers", -1)
    if player.level >= 66 then
        dncChance(effects, DNC_EFFECT_THREEFOLD_FAN_DANCE, 0.5)
    end

    return effects
end
