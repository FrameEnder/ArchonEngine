-- Aetherflow (Lv. 45): restores 20 % of maximum MP; three Aetherflow.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:restore_mp(math.floor(player.max_mp / 5))
    effects:gain_aetherflow(3)

    return effects
end
