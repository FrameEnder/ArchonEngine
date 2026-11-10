-- Brand Of Purgatory (Lv. 80): 240 to the target and round it; only in Firebird Trance. The demi-summon's own attack comes with it (Global.lua's smnDemiHit).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 240, TRAITS_ACN_MAIM_AND_MEND)))
    smnDemiHit(effects, player)

    return effects
end
