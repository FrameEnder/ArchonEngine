-- An action of Haukke Manor's the last boss's 14506 (territory 1040, with Duty Support or the Duty Finder): on an ally
-- (the boss): status 151 at 65, lost 2.9 and 8.8 s later (6 s here). From Bob's Chronofoil captures 5d9c7d7f and
-- e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its area and cast are the Action
-- sheet's.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(151, 65, 6.0)

    return effects
end
