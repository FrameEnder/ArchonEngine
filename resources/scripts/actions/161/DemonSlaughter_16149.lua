-- Demon Slaughter (Lv. 40): 100 to all enemies near, 160 after Demon Slice with a Cartridge.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        gnbDamage(effects, player, 160)
        gnbAddCartridges(effects, player, 1)
    else
        gnbDamage(effects, player, 100)
    end

    return effects
end
