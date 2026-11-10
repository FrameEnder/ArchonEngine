-- Automaton Queen (Lv. 80): the queen, 12 s, for the Battery (all of it, 50 at least): Arm Punches, Roller Dash, Pile Bunker and Crowned Collider (86), 1,570 in all at 50 Battery, twice that at 100 (the job guide's numbers), as six hits with the next weaponskills (no pets).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local battery = player:gauge("battery")
    local each = player.level >= 86 and 262 or 197
    petSummon(effects, 6, math.floor(each * battery / 50))
    effects:gauge_set("battery", 0)

    return effects
end
