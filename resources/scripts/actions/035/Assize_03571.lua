-- Assize (Lv. 56)
-- Damages all enemies and heals the party within 20 yalms of the caster.
DAMAGE_POTENCY = 400
HEAL_POTENCY = 400
MP_RESTORED = 500 -- 5% of maximum MP (10,000)

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, DAMAGE_POTENCY))
    effects:heal(whmHeal(player, HEAL_POTENCY))
    effects:restore_mp(MP_RESTORED)

    return effects
end
