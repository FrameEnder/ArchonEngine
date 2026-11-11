-- A quest enemy's cast (BNpcBase 2481, quest 1192, GaiUse118): 2.7 s, median 407 against the 165 of the 966 it
-- casts instead of auto-attacking (Connor's Chronofoil capture af05916c), so about 250 of the auto-attack's 100.
POTENCY = 250

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
