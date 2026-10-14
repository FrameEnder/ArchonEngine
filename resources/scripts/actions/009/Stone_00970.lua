-- Floating Eye's attack: cast instead of auto-attacking.
-- Potency is relative to the enemy auto-attack (100), from what Kawari Bestiary recorded on retail.
-- Worked out like enemies' physical attacks, so it's scaled the same way (see NPC_DAMAGE_TO_PLAYERS_PERCENT.)
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
