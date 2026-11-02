-- An action of Haukke Manor's the 14698 (territory 1040, with Duty Support or the Duty Finder): a gaze every 30.7 s,
-- missed every time (five): what it does to those facing it wasn't seen. From Bob's Chronofoil captures 5d9c7d7f and
-- e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its area and cast are the Action
-- sheet's.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gaze()

    return effects
end
