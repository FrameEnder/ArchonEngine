-- Astral Impulse (Lv. 58): 500; only in Dreadwyrm Trance. The demi-summon's own attack comes with it (Global.lua's smnDemiHit).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 500, TRAITS_ACN_MAIM_AND_MEND)))
    smnDemiHit(effects, player)

    return effects
end
