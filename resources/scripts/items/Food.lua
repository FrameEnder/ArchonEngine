-- Food (ItemAction 844-846): Well Fed (status 48), its param the ItemFood row (Data[1], plus 10000 for HQ food) and its
-- duration Data[2] seconds. `arg` is the param in its low 16 bits and the duration in its high 16 bits. The stat bonuses
-- come from the ItemFood row while the status lasts (`GameData::food_bonuses`, the player's connection.)
EFFECT_WELL_FED = 48

function doAction(player, arg)
    effects = EffectsBuilder()
    -- Eating again adds the food's time to what's left (retail, Bob's Chronofoil captures: 1800 s left + 1800 → 3597,
    -- 2400 + 2400 → 4797), here up to twice the food's time.
    local duration = arg // 65536
    local left = player:effect_time_left(EFFECT_WELL_FED) or 0
    effects:gain_effect_self(EFFECT_WELL_FED, arg % 65536, math.min(left + duration, 2 * duration))

    return effects
end
