-- Pitch Perfect (Lv. 52): 100, 220 or 360 for one, two or three Repertoire (all spent), 50 % less round the target; only in the Minuet.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local repertoire = player:gauge("repertoire")
    local potency = ({ 100, 220, 360 })[math.max(1, math.min(repertoire, 3))]
    effects:aoe_falloff(50)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, potency, TRAITS_ARC_ACTION_DAMAGE)))
    effects:gauge_set("repertoire", 0)

    return effects
end
