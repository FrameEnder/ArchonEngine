-- HP potions (ItemAction 847): Data[0] is a percentage of max HP and Data[1] its cap (Potion 32%/160, Hi-Potion 32/340).
-- `arg` is the percentage in its high 16 bits and the cap in its low 16 bits. HQ values (DataHQ) aren't used yet.
function doAction(player, arg)
    local percent = arg // 65536
    local cap = arg % 65536
    effects = EffectsBuilder()
    effects:heal(math.min(math.floor(player.max_hp * percent / 100), cap))

    return effects
end
