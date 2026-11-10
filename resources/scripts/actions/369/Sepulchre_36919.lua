-- Sepulchre (Lv. 76): 540, only with Sepulchre Ready; MP.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 540))
    effects:consume_effect(PLD_EFFECT_SEPULCHRE_READY)
    effects:restore_mp(400)

    return effects
end
