-- Plaincracker: Spirithold Broken's clay golem. A 6 yalm circle around itself (the Action sheet.)
-- On retail it hit a level 9 conjurer for 21-23, about 6.2x the golem's auto-attack, as recorded with Kawari Recorder.
-- Potency is relative to the enemy auto-attack (100).
POTENCY = 620

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
