-- Food (ItemAction 844-846): Well Fed (status 48), its param the ItemFood row (Data[1]) and its duration Data[2] seconds.
-- `arg` is the row in its low 16 bits and the duration in its high 16 bits.
-- NOTE: the stat bonuses from the ItemFood row aren't applied by the server yet; the status is.
EFFECT_WELL_FED = 48

function doAction(player, arg)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_WELL_FED, arg % 65536, arg // 65536)

    return effects
end
