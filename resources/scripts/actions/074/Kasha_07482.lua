-- Kasha (Lv. 40): 160, 370 after Shifu (or under Meikyo Shisui, with Fuka) with Ka and 10 Kenki.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samStep(effects, player, in_combo, tierPotency(player, { { 94, 160 }, { 84, 110 } }), 370, 4, 10, player:get_effect(SAM_EFFECT_MEIKYO_SHISUI) ~= nil and SAM_EFFECT_FUKA or nil)

    return effects
end
