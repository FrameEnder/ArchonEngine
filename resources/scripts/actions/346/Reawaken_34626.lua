-- Reawaken (Lv. 90): 750 to the first enemy near, 75 % less to the rest; Reawakened with four Anguine Tribute (five from 96), 30 s; 50 Serpent Offerings (none when Ready to Reawaken).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, 750)
    if player:get_effect(VPR_EFFECT_READY_TO_REAWAKEN) ~= nil then
        effects:consume_effect(VPR_EFFECT_READY_TO_REAWAKEN)
    else
        effects:gauge_add("offerings", -50)
    end
    effects:gain_effect_self(VPR_EFFECT_REAWAKENED, 0, 30.0)
    effects:gauge_set("tribute", player.level >= 96 and 5 or 4)

    return effects
end
