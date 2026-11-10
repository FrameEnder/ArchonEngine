-- Six Sided Star (Lv. 80): 780, 80 more for each open chakra, which it closes; faster for 5 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local chakra = player:gauge("chakra")
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 780 + 80 * chakra))
    effects:gauge_set("chakra", 0)
    effects:gain_effect_self(MNK_EFFECT_SIX_SIDED_STAR, 0, 5.0)

    return effects
end
