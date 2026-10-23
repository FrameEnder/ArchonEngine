-- Denn's (Sastasha's last boss) action, an enemy's only. What it did on retail is in its timeline's `recorded_amounts`
-- (resources/timelines/Denn_*.json, Bob's Chronofoil captures of Sastasha), which scales this hit to it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, physicalDamage(player, POTENCY))

    return effects
end
