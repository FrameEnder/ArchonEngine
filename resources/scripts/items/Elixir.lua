-- Elixirs (ItemAction 849): HP and MP. `arg` is the HP cap in its low 16 bits and the MP cap in its high 16 bits.
-- NOTE: the percentages of max HP/MP aren't used (the script can't see them), only the caps.
function doAction(player, arg)
    effects = EffectsBuilder()
    effects:heal(arg % 65536)
    effects:restore_mp(arg // 65536)

    return effects
end
