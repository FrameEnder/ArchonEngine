-- Ethers (ItemAction 848): Data[0] is a percentage of max MP and Data[1] its cap (Ether 6%/600).
-- `arg` is the percentage in its high 16 bits and the cap in its low 16 bits. HQ values aren't used yet.
function doAction(player, arg)
    local percent = arg // 65536
    local cap = arg % 65536
    effects = EffectsBuilder()
    effects:restore_mp(math.min(math.floor(player.max_mp * percent / 100), cap))

    return effects
end
