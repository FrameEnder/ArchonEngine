-- Denn's (Sastasha's last boss), a 3.2 s cast action, an enemy's only. What it did on retail is in its timeline's `recorded_amounts`
-- (resources/timelines/Denn_*.json, Bob's Chronofoil captures of Sastasha), which scales this hit to it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
