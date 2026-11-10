-- Ouroboros (Lv. 96): 1,150 to the target, 75 % less round it; Reawaken over.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(75)
    vprDamage(effects, player, 1150)
    effects:gauge_set("tribute", 0)
    if player:get_effect(VPR_EFFECT_REAWAKENED) ~= nil then
        effects:consume_effect(VPR_EFFECT_REAWAKENED)
    end

    return effects
end
