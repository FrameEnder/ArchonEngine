-- An action quest battle NPCs use (the Akoman in quest battle 17, BNpcBase 301): a gaze. Everyone it catches who is
-- facing it is paralysed (status 17, param 0: see QB56Sprite_01118); anyone looking away is missed (Bob's Chronofoil capture df6f5af9, three
-- runs: 13 casts, every target within 58 degrees of facing it paralysed, every one 94 degrees or more away missed;
-- the paralysis lost 16.1 s later where nothing cut it short). Its area is the Action sheet's.
-- The floating eye (BNpcBase 182) opens with it too (Power of Deduction's, four retail fights; FloatingEye_182.) What
-- paralysis then does is the server's (server/paralysis.rs.)
local STATUS = 17
local PARAM = 0
local DURATION = 16.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gaze()
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
