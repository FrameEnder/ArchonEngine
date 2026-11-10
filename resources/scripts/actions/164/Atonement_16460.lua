-- Atonement (Lv. 76): 460, only with Atonement Ready; gives Supplication Ready (30 s) and MP.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 460))
    effects:consume_effect(PLD_EFFECT_ATONEMENT_READY)
    effects:gain_effect_self(PLD_EFFECT_SUPPLICATION_READY, 0, 30.0)
    effects:restore_mp(400)

    return effects
end
