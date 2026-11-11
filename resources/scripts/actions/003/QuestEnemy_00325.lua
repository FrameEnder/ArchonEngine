-- A quest enemy's cast (BNpcBase 2485, quest 1190, GaiUse116): 2.2 s, about every 12.7 s, median 286 against an
-- auto-attack's 98 on the same NPCs (Connor's Chronofoil capture af05916c), so about 290 of the auto-attack's 100.
POTENCY = 290

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
