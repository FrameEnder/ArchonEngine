-- The Porta Decumana's second boss's 29023: a 4.7 s cast on its target: 920 to 1285 to Duty Support's tank (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.) The timeline's
-- `recorded_amounts` scale it.

local POTENCY = 500

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
