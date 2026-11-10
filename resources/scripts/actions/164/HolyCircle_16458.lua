-- Holy Circle (Lv. 72): 100 round the Paladin, 250 under Divine Might, 350 under Requiescat; heals it 400.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, pldMagicPotency(effects, player, 100, 250, 350)))
    effects:heal_self(healAmount(player, 400))

    return effects
end
