-- Flash, which gladiators on your side use in quest battles (That Old Familiar Feeling's Aldis, BNpcBase 629, and The
-- Face of Thal's gladiators, 638 and 868): no damage; every enemy within 5 yalms of the user (the Action sheet's area)
-- gets enmity and status 15 at 236 (Bob's Chronofoil capture 941895d2 of 2026-11-06, quest battles 20 and 21: every
-- use, two or three caught, the effect refreshed on each use.) What 15 does isn't scripted, and how long it lasts
-- isn't recorded (12 s here); the enmity comes from the director's own targeting.
STATUS = 15
PARAM = 236
DURATION = 12.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
