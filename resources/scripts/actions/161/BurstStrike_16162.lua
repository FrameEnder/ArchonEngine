-- Burst Strike (Lv. 30): 420; a Cartridge; Ready to Blast (10 s) from 86.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    gnbDamage(effects, player, tierPotency(player, { { 94, 420 }, { 1, 380 } }))
    effects:gauge_add("cartridges", -1)
    if player.level >= 86 then
        effects:gain_effect_self(GNB_EFFECT_READY_TO_BLAST, 0, 10.0)
    end

    return effects
end
