-- Horoscope End (Lv. 76): heals the party near 200 (400 after a Helios; here 200).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 200)
    if player:get_effect(AST_EFFECT_HOROSCOPE) ~= nil then
        effects:consume_effect(AST_EFFECT_HOROSCOPE)
    end

    return effects
end
