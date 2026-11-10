-- Chain Saw (Lv. 90): 660 to the first enemy in a line, 25 % less to the rest; 20 Battery; Excavator Ready (30 s) from 96.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    mchShot(effects, player, 660)
    mchHeat(effects, player, 0, 20)
    if player.level >= 96 then
        effects:gain_effect_self(MCH_EFFECT_EXCAVATOR_READY, 0, 30.0)
    end

    return effects
end
