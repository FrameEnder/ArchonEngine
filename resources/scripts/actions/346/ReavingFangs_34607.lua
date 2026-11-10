-- Reaving Fangs (Lv. 10): 200, 300 with Honed Reavers; Honed Steel (60 s).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    vprDamage(effects, player, tierPotency(player, { { 74, 200 }, { 1, 150 } }) + vprBonus(effects, player, VPR_EFFECT_HONED_REAVERS, 100))
    effects:gain_effect_self(VPR_EFFECT_HONED_STEEL, 0, 60.0)

    return effects
end
