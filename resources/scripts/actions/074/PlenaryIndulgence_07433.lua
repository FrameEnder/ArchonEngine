-- Plenary Indulgence (Lv. 70)
-- Grants Confession to the caster and party members within 30 yalms.
-- NOTE: The status is applied, but damage reduction and the bonus healing aren't implemented yet.
DURATION = 10.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(EFFECT_CONFESSION, 0, DURATION)

    return effects
end
