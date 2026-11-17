-- Quest battle 82's master's action on its own creature (BNpcBase 2218: a 4.2 s cast, once, the script's:
-- content/questbattle/82.lua): retail's took all of the creature's health (12945) and gave the master 5209, which
-- brought it back to full; the creature faded out 1.3 s later (Bob's Chronofoil capture a75e1390 of 2026-11-17).
-- Here it does nothing itself: the script takes the creature away and the master's health isn't restored.
function doAction(player, in_combo)
    return EffectsBuilder()
end
