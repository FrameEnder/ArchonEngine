-- Lucid Dreaming (Lv. 14) - role action
-- Potency 55 restores 550 MP every 3 seconds.
MP_PER_TICK = 550
DURATION = 21.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:restore_mp_over_time_self(EFFECT_LUCID_DREAMING, DURATION, MP_PER_TICK)

    return effects
end
