-- Syphon Strike (Lv. 2): 240, 380 after Hard Slash with 600 MP back.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        drkDamage(effects, player, 380)
        effects:restore_mp(600)
    else
        drkDamage(effects, player, tierPotency(player, { { 94, 240 }, { 84, 120 } }))
    end
    drkWeaponskill(effects, player)

    return effects
end
