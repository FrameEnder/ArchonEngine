-- Glare IV (Lv. 92)
-- Hits the target and enemies within 5 yalms of it. Everyone after the first takes 40% less damage.
-- Can only be used while Sacred Sight is active (granted by Presence of Mind), and uses up one stack.
POTENCY = 640

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, whmDamage(player, POTENCY))
    effects:aoe_falloff(40)
    effects:consume_effect_stack(EFFECT_SACRED_SIGHT)

    return effects
end
