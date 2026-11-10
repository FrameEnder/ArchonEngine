-- Verraise (the Red Mage's Raise): offers a dead party member a revive (status 148, 60 s), as Raise does (Raise_00125.lua).
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:offer_revive()

    return effects
end
