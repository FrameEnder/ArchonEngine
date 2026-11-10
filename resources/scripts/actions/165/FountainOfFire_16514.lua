-- Fountain Of Fire (Lv. 80): 580; only in Firebird Trance. The demi-summon's own attack comes with it (Global.lua's smnDemiHit).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 580, TRAITS_ACN_MAIM_AND_MEND)))
    smnDemiHit(effects, player)

    return effects
end
