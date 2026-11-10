-- Mirage Dive (Lv. 68): 380; only with Dive Ready.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, 380))
    effects:consume_effect(DRG_EFFECT_DIVE_READY)

    return effects
end
