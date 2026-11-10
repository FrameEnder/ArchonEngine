-- Scatter (Lv. 15): 120 to the target and round it (170 with Acceleration, which it spends); 3 White and 3 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local accelerated = player:get_effect(RDM_EFFECT_ACCELERATION) ~= nil
    if accelerated then
        effects:consume_effect(RDM_EFFECT_ACCELERATION)
    end
    rdmMagic(effects, player, accelerated and 170 or 120)
    rdmMana(effects, 3, 3)
    rdmDualcast(effects, player, accelerated)

    return effects
end
