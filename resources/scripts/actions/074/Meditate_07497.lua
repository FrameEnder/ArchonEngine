-- Meditate (Lv. 60): Kenki over 15 s held (here: 50 at once) and a Meditation stack in combat.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    samKenki(effects, player, 50)
    effects:gauge_add("meditation", 1)

    return effects
end
