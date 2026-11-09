-- An action quest battle NPCs use on their side (quest battle 62's conjurer, BNpcBase 1651): status 151 on the one
-- it's cast on (2.7 s cast; the player, the escort, the other ally or itself; Connor's Chronofoil capture e834e436
-- and Bob's 689b15da: the ActionEffect). Its length wasn't seen end: 30 s is a guess. Not verified for players.
local STATUS = 151
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
