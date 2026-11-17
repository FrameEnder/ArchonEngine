-- The Ultimate Weapon's last battle's boss's 29340: a 3.7 s cast on the player: 553 to 1019, and status 3122 (param
-- 226) for 13 s; 29341 follows 2.0 s after it lands and 29342 1.9 s after that in the second part (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) What the status does isn't
-- known.
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
