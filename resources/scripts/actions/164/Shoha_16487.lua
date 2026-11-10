-- Shoha (Lv. 80): 640 to the first enemy in a line, 40 % less to the rest; three Meditation stacks.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(40)
    samDamage(effects, player, 640)
    effects:gauge_set("meditation", 0)

    return effects
end
