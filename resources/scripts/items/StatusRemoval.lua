-- Status cures (ItemAction 842): removes the status in Data[0] (Antidote 18, Eye Drops 15, Echo Drops 7.)
function doAction(player, arg)
    effects = EffectsBuilder()
    effects:lose_effect(arg, 0)

    return effects
end
