-- An action of Haukke Manor's the pair's caster (BNpcBase 273) (territory 1040, with Duty Support or the Duty Finder):
-- on itself every 31.8 s: status 198 at 30 for 20.5 s (lost 20.5 s later all ten times), nothing else. From Bob's
-- Chronofoil captures 5d9c7d7f and e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its
-- area and cast are the Action sheet's.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(198, 30, 20.5)

    return effects
end
