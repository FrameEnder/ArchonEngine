-- Clemency (Lv. 58): heals 1,000; the Paladin half that when it heals a party member.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(healAmount(player, 1000))

    return effects
end
