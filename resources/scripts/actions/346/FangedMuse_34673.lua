-- Fanged Muse (Lv. 96): 800 to the target, 70 % less round it; the Madeen Portrait, the depictions gone.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    pctMuse(effects, player, 8, 0)

    return effects
end
