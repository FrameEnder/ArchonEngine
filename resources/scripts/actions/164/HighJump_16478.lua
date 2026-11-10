-- High Jump (Lv. 74): 400; Dive Ready (15 s).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 400))
    effects:gain_effect_self(DRG_EFFECT_DIVE_READY, 0, 15.0)

    return effects
end
