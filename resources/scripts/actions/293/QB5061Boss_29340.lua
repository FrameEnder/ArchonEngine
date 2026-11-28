-- The Ultimate Weapon's last battle's boss's 29340: a 3.7 s cast on the player: 553 to 1019, and status 3122 (param
-- 226) for 13 s; 29341 follows 2.0 s after it lands and 29342 1.9 s after that in the second part (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) What the status does isn't
-- known.
local POTENCY = 300
local STATUS = 3122
-- (Retail's effect: `0E E2 EC 01 00 00 32 0C`, first, its parameter 1; never once the ally's 3123 was on the player,
-- Bob's Chronofoil captures 9ba7d981, 6320ae56 and 50e72631.)
local STATUS_PARAM = 1
local STATUS_BYTES = { 0xE2, 0xEC }
local STATUS_NOT_WITH = 3123
local DURATION = 13.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(STATUS, STATUS_PARAM, DURATION, STATUS_BYTES[1], STATUS_BYTES[2])
    effects:spare_bearers(STATUS, STATUS_NOT_WITH)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
