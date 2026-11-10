-- Microcosmos (Lv. 90): heals 200 (with half the damage taken under Macrocosmos: not counted); Macrocosmos over.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    astHeal(effects, player, 200)
    if player:get_effect(AST_EFFECT_MACROCOSMOS) ~= nil then
        effects:consume_effect(AST_EFFECT_MACROCOSMOS)
    end

    return effects
end
