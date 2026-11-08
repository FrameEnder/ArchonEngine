-- Garuda's 659 (the Howling Eye, 2.2 s, round her): a magic hit and status 2 (a stun), lost 4.2 s later (one hit, 697).
-- From Bob's Chronofoil captures of 2026-11-08 (6a5244ec, 09ecf8e7, 6c495fc4); the hit is scaled to the timeline's
-- recorded amount.
local POTENCY = 100
local STATUS = 2
local PARAM = 0
local DURATION = 4.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
