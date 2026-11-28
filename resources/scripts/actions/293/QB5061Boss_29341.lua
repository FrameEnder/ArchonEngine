-- The Ultimate Weapon's last battle's boss's 29341: no cast, on the player, 2.0 s after 29340 lands: 816 to 1067, and
-- status 3122 again (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.)
local POTENCY = 300
local STATUS = 3122
-- (Retail's effect: `0E E2 EC 02 00 00 32 0C`, first, its parameter 2; never once the ally's 3123 was on the player,
-- Bob's Chronofoil captures 9ba7d981, 6320ae56 and 50e72631.)
local STATUS_PARAM = 2
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
