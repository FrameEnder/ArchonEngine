-- Carve And Spit (Lv. 60): 540; heals 500 and 600 MP back.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 540)
    effects:heal_self(healAmount(player, 500))
    effects:restore_mp(600)

    return effects
end
