-- Verthunder I I I (Lv. 82): 440; 6 Black Mana; Verfire Ready half the time from 26 (always with Acceleration, which it spends).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, 440)
    rdmMana(effects, 0, 6)
    local accelerated = player:get_effect(RDM_EFFECT_ACCELERATION) ~= nil
    if accelerated then
        effects:consume_effect(RDM_EFFECT_ACCELERATION)
    end
    if player.level >= 26 and (accelerated or math.random() < 0.5) then
        effects:gain_effect_self(RDM_EFFECT_VERFIRE_READY, 0, 30.0)
    end
    rdmDualcast(effects, player, accelerated)

    return effects
end
