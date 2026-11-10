-- Zesho Meppo (Lv. 96): 700, 850 under Meisui; 50 Ninki; only with Higi.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 700
    if player:get_effect(NIN_EFFECT_MEISUI) ~= nil then
        potency = 850
        effects:consume_effect(NIN_EFFECT_MEISUI)
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, potency))
    effects:gauge_add("ninki", -50)
    effects:consume_effect(NIN_EFFECT_HIGI)

    return effects
end
