-- Fan Dance I I (Lv. 50): 100 to all enemies near; a Feather; Threefold Fan Dance half the time from 66.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    dncDamage(effects, player, 100)
    effects:gauge_add("feathers", -1)
    if player.level >= 66 then
        dncChance(effects, DNC_EFFECT_THREEFOLD_FAN_DANCE, 0.5)
    end

    return effects
end
