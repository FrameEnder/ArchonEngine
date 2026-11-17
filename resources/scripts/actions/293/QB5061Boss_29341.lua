-- The Ultimate Weapon's last battle's boss's 29341: no cast, on the player, 2.0 s after 29340 lands: 816 to 1067, and
-- status 3122 again (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.)
local POTENCY = 300
local STATUS = 3122
local STATUS_PARAM = 226
local DURATION = 13.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STATUS, STATUS_PARAM, DURATION)

    return effects
end
