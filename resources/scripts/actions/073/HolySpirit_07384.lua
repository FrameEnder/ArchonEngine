-- Holy Spirit (Lv. 64): 400, 500 under Divine Might, 700 under Requiescat (Divine Might first); heals the Paladin 400 from 84.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, pldMagicPotency(effects, player, 400, 500, 700)))
    pldSpellHeal(effects, player)

    return effects
end
