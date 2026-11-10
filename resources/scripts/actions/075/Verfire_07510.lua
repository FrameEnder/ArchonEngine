-- Verfire (Lv. 26): 380; 5 Black Mana; Verfire Ready spent.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    rdmMagic(effects, player, tierPotency(player, { { 94, 380 }, { 84, 340 }, { 62, 300 }, { 1, 270 } }))
    rdmMana(effects, 0, 5)
    effects:consume_effect(RDM_EFFECT_VERFIRE_READY)
    rdmDualcast(effects, player)

    return effects
end
