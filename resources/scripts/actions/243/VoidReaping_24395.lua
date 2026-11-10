-- Void Reaping (Lv. 80): 580 (640 with Enhanced Void Reaping); Enhanced Cross Reaping (30 s); a Lemure Shroud for a Void Shroud.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if player:get_effect(RPR_EFFECT_ENHANCED_VOID_REAPING) ~= nil then
        rprDamage(effects, player, 640)
        effects:consume_effect(RPR_EFFECT_ENHANCED_VOID_REAPING)
    else
        rprDamage(effects, player, 580)
    end
    effects:gain_effect_self(RPR_EFFECT_ENHANCED_CROSS_REAPING, 0, 30.0)
    rprReaping(effects, player)

    return effects
end
