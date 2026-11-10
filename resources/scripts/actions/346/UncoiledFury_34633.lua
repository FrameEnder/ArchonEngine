-- Uncoiled Fury (Lv. 82): 680 to the target, 75 % less round it; a Rattling Coil; Poised for Twinfang (60 s) from 92.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, 680)
    effects:gauge_add("coils", -1)
    if player.level >= 92 then
        effects:gain_effect_self(VPR_EFFECT_POISED_FOR_TWINFANG, 0, 60.0)
    end

    return effects
end
