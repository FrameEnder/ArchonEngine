-- Verstone (Lv. 30): 380; 5 White Mana; Verstone Ready spent.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, tierPotency(player, { { 94, 380 }, { 84, 340 }, { 62, 300 }, { 1, 270 } }))
    rdmMana(effects, 5, 0)
    effects:consume_effect(RDM_EFFECT_VERSTONE_READY)
    rdmDualcast(effects, player)

    return effects
end
