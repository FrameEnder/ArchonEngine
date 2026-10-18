-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
POTENCY = 260

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
