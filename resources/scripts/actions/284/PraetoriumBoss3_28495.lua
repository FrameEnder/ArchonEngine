-- The Praetorium's last boss's 28495: a 4.7 s cast on everyone: 900 to a player, 828 to a member (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18, five runs.) Its area
-- is the Action sheet's; the timeline's `recorded_amounts` scale it.

local POTENCY = 350

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
