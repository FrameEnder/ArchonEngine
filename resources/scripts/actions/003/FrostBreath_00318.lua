-- Anole and Lindwurm. Hit about 3x their auto-attack on retail (Kawari Recorder), with no status.
-- Potency is relative to the enemy auto-attack (100).
POTENCY = 300

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BREATH, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
