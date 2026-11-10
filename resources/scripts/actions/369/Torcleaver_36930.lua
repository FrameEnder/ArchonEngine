-- Torcleaver (Lv. 96): 820 after Comeuppance. 200 MP back (the guide says "restores MP"; 200 as retail's tooltip had it in 7.0).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drkDamage(effects, player, 820)
    effects:consume_effect_stack(DRK_EFFECT_ENHANCED_DELIRIUM)
    effects:restore_mp(200)
    drkWeaponskill(effects, player)

    return effects
end
