-- The NPC version of Shield Bash, used by The Face of Thal's enemy gladiators (quest battle 21, BNpcBase 635) on the
-- gladiator on your side: a hit and status 2 (stun) at 0 (Bob's Chronofoil capture 941895d2 of 2026-11-06: once each,
-- once missed). How much it does comes from the NPC's timeline (recorded_amounts: 3), not from this potency; how long
-- the stun lasts isn't recorded (Shield Bash's 6 s here.)
-- TODO: the potency and the stun's length aren't verified.
POTENCY = 100
STUN_DURATION = 6.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(EFFECT_STUN, 0, STUN_DURATION)

    return effects
end
