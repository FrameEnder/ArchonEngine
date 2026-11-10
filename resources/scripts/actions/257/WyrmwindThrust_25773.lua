-- Wyrmwind Thrust (Lv. 90): 440 to the first enemy in a line, 50 % less to the rest; spends two Firstminds' Focus.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 440))
    effects:gauge_add("firstminds_focus", -2)

    return effects
end
