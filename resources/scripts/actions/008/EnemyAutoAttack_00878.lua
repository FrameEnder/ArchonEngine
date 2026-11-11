-- An enemy auto-attack (BNpcBase 2485, one of quest 1190's, GaiUse116): every 3 s, median 52 to the player and 98
-- to the Duty Support or allied NPCs in Connor's Chronofoil capture af05916c (2026-11-07/09). Its damage type isn't
-- recorded; slashing is a guess. The timeline's recorded amounts come first where they apply.
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
