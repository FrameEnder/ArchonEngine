-- An action of Haukke Manor's the manor's 276s (BNpcBase 276) (territory 1040, with Duty Support or the Duty Finder):
-- status 18 on its target every 31.8 s, no damage: retail's param was 30 most often (15 of 31; 24-91 the rest); when
-- it's lost wasn't seen, 15 s is a guess. From Bob's Chronofoil captures 5d9c7d7f and e7580e0d (2026-11-02): one run
-- with Duty Support and three Duty Finder runs. Its area and cast are the Action sheet's.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(18, 30, 15.0)

    return effects
end
