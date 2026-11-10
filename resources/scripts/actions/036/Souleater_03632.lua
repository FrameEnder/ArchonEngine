-- Souleater (Lv. 26): 260, 480 after Syphon Strike with 300 healed and 20 Blood (62).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        drkDamage(effects, player, 480)
        effects:heal_self(healAmount(player, 300))
        if player.level >= 62 then
            effects:gauge_add("blood", 20)
        end
    else
        drkDamage(effects, player, tierPotency(player, { { 94, 260 }, { 84, 140 } }))
    end
    drkWeaponskill(effects, player)

    return effects
end
