-- Quest battle 62's 1655s (BNpcBase 1655): a 2.2 s cast on one target every 11 s, 396-424 on an ally and status 17
-- (parameter 50) on it (Connor's Chronofoil capture e834e436 and Bob's 689b15da: the ActionEffect), scaled to the
-- timeline's recorded amounts. The status's length wasn't seen: 10 s is a guess.
local POTENCY = 250
local STATUS = 17
local PARAM = 50
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, PARAM, DURATION)

    return effects
end
