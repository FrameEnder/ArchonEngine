-- Solid Barrel (Lv. 26): 240, 460 after Brutal Shell with a Cartridge (30).
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        gnbDamage(effects, player, 460)
        if player.level >= 30 then
            gnbAddCartridges(effects, player, 1)
        end
    else
        gnbDamage(effects, player, tierPotency(player, { { 94, 240 }, { 84, 140 } }))
    end

    return effects
end
