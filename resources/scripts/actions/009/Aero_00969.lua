-- Wind Sprite's attack (Trial by Wind's quest battle 54): on retail it casts this instead of auto-attacking, every
-- 3.1 s (Bob's Chronofoil capture b23b91ef, 2026-10-26: 7-11 on players, the median 8; its timeline's
-- `recorded_amounts` scales the hit to retail's.)
-- Worked out like enemies' physical attacks, so it's scaled the same way (see NPC_DAMAGE_TO_PLAYERS_PERCENT), as the
-- other sprites' (Water_00971.)
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
