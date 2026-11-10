-- Salt And Darkness (Lv. 86): 500 to the first enemy in the Salted Earth, 25 % less to the rest (here: the target).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    drkDamage(effects, player, 500, DAMAGE_TYPE_MAGIC)

    return effects
end
