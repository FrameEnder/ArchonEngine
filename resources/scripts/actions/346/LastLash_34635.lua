-- Last Lash (Lv. 60): 120 to all enemies near after Jagged or Bloodied Maw.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, 120)

    return effects
end
