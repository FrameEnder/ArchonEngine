-- Liturgy Of The Bell (Lv. 90)
-- NOTE: Liturgy of the Bell is ground-targeted, and Kawari doesn't handle ground-targeted action requests yet, so this won't run until it does.
-- The stacks are granted, but healing when taking damage isn't implemented.
STACKS = 5
DURATION = 20.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_LITURGY_OF_THE_BELL, STACKS, DURATION)

    return effects
end
