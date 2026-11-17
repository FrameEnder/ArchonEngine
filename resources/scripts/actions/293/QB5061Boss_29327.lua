-- The Ultimate Weapon's last battle's boss's 29327: a 3.7 s cast on himself: 1031 to 1230 to the player (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) Its area is the Action
-- sheet's; the timeline's `recorded_amounts` scale it.
local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
