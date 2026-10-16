-- Enemy auto-attack (the kind the Gods' Quiver bows, Galfrid and the hedge knights use in To Guard a Guardian
-- and Leia's Legacy.) Potency is relative to the enemy auto-attack (100).
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
