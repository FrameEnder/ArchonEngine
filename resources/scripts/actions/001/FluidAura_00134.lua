-- The NPC version of Fluid Aura, used by The Face of Thal's enemy conjurers (quest battle 21, BNpcBase 636): no damage,
-- status 13 (bind) at 0 on its target (Bob's Chronofoil capture 941895d2 of 2026-11-06: once, on the gladiator on your
-- side.) How long it lasts isn't recorded (6 s here). What 13 does isn't scripted.
STATUS = 13
PARAM = 0
DURATION = 6.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
