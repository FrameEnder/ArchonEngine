-- Raiden Thrust (Lv. 76): 320; one Firstminds' Focus; only with Draconian Fire.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 320
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    effects:consume_effect(DRG_EFFECT_DRACONIAN_FIRE)
    effects:gauge_add("firstminds_focus", 1)

    return effects
end
