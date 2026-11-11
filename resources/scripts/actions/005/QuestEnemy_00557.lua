-- A quest enemy's attack (BNpcBase 2891, quest 1359, GaiUse215): instant, about every 8.2 s, median 94 against an
-- auto-attack's 188 (Connor's Chronofoil capture af05916c), so about 50 of the auto-attack's 100.
POTENCY = 50

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
