-- Bhavacakra (Lv. 68): 400, 550 under Meisui; 50 Ninki.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 400
    if player:get_effect(NIN_EFFECT_MEISUI) ~= nil then
        potency = 550
        effects:consume_effect(NIN_EFFECT_MEISUI)
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, potency))
    effects:gauge_add("ninki", -50)

    return effects
end
