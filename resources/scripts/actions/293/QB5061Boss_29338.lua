-- The Ultimate Weapon's last battle's boss's 29338: a 3.7 s cast on himself: status 661 on him, which the server took
-- off 9.1 s later (ActorControl LoseEffect: its duration here) (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) What the status does isn't known.
local STATUS = 661
local DURATION = 9.1

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, 0, DURATION)

    return effects
end
