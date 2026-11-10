-- Shake It Off (Lv. 68): a barrier of 15 % of max HP for the party near (30 s; barriers don't absorb yet), 100 over 15 s, and a heal of 300 from 76.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(1457, 0, 30.0)
    effects:gain_effect(2108, 0, 15.0)
    if player.level >= 76 then
        effects:heal(healAmount(player, 300))
    end

    return effects
end
