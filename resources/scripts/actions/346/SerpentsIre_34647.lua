-- Serpents Ire (Lv. 86): a Rattling Coil; Ready to Reawaken (30 s) from 90.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gauge_add("coils", 1)
    if player.level >= 90 then
        effects:gain_effect_self(VPR_EFFECT_READY_TO_REAWAKEN, 0, 30.0)
    end

    return effects
end
