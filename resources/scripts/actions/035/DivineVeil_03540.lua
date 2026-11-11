-- Divine Veil (Lv. 56): a barrier of 10 % of max HP for the party near (30 s) and a heal of 400 (Enhanced Divine Veil, 88).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(1362, 0, 30.0)
    effects:barrier_percent(1362, 10)
    if player.level >= 88 then
        effects:heal(healAmount(player, 400))
    end

    return effects
end
