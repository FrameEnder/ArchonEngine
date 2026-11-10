-- Fated Circle (Lv. 72): 300 to all enemies near; a Cartridge; Ready to Raze (10 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, 300)
    effects:gauge_add("cartridges", -1)
    if player.level >= 96 then
        effects:gain_effect_self(GNB_EFFECT_READY_TO_RAZE, 0, 10.0)
    end

    return effects
end
