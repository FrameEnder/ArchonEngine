-- Stalwart Soul (Lv. 40): 120 to all enemies near, 160 after Unleash with 600 MP back and 20 Blood (62).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        drkDamage(effects, player, 160, DAMAGE_TYPE_MAGIC)
        effects:restore_mp(600)
        if player.level >= 62 then
            effects:gauge_add("blood", 20)
        end
    else
        drkDamage(effects, player, 120, DAMAGE_TYPE_MAGIC)
    end
    drkWeaponskill(effects, player)

    return effects
end
