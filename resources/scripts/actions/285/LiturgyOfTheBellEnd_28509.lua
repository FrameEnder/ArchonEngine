-- Liturgy of the Bell, ended early (Lv. 90): the bell heals the White Mage and the party near 200 for each stack left,
-- then goes.
-- Potencies: the official job guide (7.x), at level 100.
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local bell = player:get_effect(EFFECT_LITURGY_OF_THE_BELL)
    if bell ~= nil and bell.param > 0 then
        effects:heal(whmHeal(player, 200 * bell.param))
    end
    effects:consume_effect(EFFECT_LITURGY_OF_THE_BELL)
    return effects
end
