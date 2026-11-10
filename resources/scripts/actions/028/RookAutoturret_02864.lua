-- Rook Autoturret (Lv. 40): the turret, 9 s, for the Battery (all of it, 50 at least): about four Volley Fires and Rook Overload, 35-70 and 160-320 as the Battery is 50-100, as four hits with the next weaponskills (no pets).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local battery = player:gauge("battery")
    petSummon(effects, 4, math.floor(84 * battery / 50))
    effects:gauge_set("battery", 0)

    return effects
end
