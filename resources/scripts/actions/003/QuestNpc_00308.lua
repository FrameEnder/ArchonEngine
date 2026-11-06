-- Paralyze as Three for Three's ambush's caster uses it (BNpcBase 726, quest 889): a cast on the player that gives
-- Paralysis (status 17, param 0) for 15 s and no damage (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06:
-- the EffectResult.) What paralysis then does is the server's (server/paralysis.rs.)
local STATUS = 17
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
