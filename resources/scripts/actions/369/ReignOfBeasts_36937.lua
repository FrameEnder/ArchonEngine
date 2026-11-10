-- Reign Of Beasts (Lv. 100): 800 to the target, 60 % less round it; Ready to Reign spent.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(60)
    gnbDamage(effects, player, 800)
    effects:consume_effect(GNB_EFFECT_READY_TO_REIGN)

    return effects
end
