-- Shadowbringer (Lv. 90): 600 to the first enemy in a line, 25 % less to the rest; only in Darkside.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(25)
    drkDamage(effects, player, 600, DAMAGE_TYPE_MAGIC)

    return effects
end
