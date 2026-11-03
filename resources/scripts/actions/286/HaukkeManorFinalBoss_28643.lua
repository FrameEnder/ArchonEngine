-- An action of Haukke Manor's the last boss (BNpcBase 14504) (territory 1040, with Duty Support or the Duty Finder):
-- every 63.1 s, on up to four: statuses 2266 and 227 for about 4 s (lost 3.6-4.0 s later), no damage. From Bob's
-- Chronofoil captures 5d9c7d7f and e7580e0d (2026-11-02): one run with Duty Support and three Duty Finder runs. Its
-- area and cast are the Action sheet's. Statuses 2266 and 227 are both Seduced: each player it lands on is walked
-- towards the boss (retail's ForcedMoveTowards [4, the boss] with the result, ActorControl 82 [1132] on them, and
-- ForcedMoveEnd and 82 [0] 3.0 s later: all five captured runs, Bob's capture 25342d59 of 2026-11-03 too), while the
-- eye beside her casts its gaze.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(2266, 0, 3.9)
    effects:gain_effect(227, 0, 3.8)
    effects:walk_to_caster(3.0, 4, 1132)

    return effects
end
