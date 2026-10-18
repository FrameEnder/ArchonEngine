-- Resurrection (Lv. 12): offers a dead party member a revive (status 148, 60 s), as Raise.
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:offer_revive()

    return effects
end
