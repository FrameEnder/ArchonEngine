-- Plentiful Harvest (Lv. 88): 720 to the first enemy in a line, 20 % less to the rest, 40 more for each Immortal Sacrifice past the first (1,000 at eight); Ideal Host and Perfectio Occulta (100), 30 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local stacks = 1
    local sacrifice = player:get_effect(RPR_EFFECT_IMMORTAL_SACRIFICE)
    if sacrifice ~= nil then
        stacks = math.max(sacrifice.param, 1)
        effects:consume_effect(RPR_EFFECT_IMMORTAL_SACRIFICE)
    end
    effects:aoe_falloff(20)
    rprDamage(effects, player, 720 + 40 * (stacks - 1))
    effects:gain_effect_self(RPR_EFFECT_IDEAL_HOST, 0, 30.0)
    if player.level >= 100 then
        effects:gain_effect_self(RPR_EFFECT_PERFECTIO_OCCULTA, 0, 30.0)
    end

    return effects
end
