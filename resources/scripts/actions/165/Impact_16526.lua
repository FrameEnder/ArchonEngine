-- Impact (Lv. 66): 210 to the target and round it (260 with Acceleration, which it spends); 3 White and 3 Black Mana.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local accelerated = player:get_effect(RDM_EFFECT_ACCELERATION) ~= nil
    if accelerated then
        effects:consume_effect(RDM_EFFECT_ACCELERATION)
    end
    rdmMagic(effects, player, accelerated and 260 or 210)
    rdmMana(effects, 3, 3)
    rdmDualcast(effects, player, accelerated)

    return effects
end
