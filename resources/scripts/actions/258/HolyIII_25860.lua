-- Holy III (Lv. 82)
-- Hits all enemies within 8 yalms of the caster.
POTENCY = 150
STUN_DURATION = 4.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))
    -- NOTE: The stun icon is applied, but enemy AI doesn't respect stuns yet.
    effects:gain_effect(EFFECT_STUN, 0, STUN_DURATION)

    return effects
end
