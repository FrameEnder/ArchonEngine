-- The Ultimate Weapon's last battle's helper's 29565: no cast, on the player, 36.7 s into the middle of the fight:
-- 9999999, which fells them; the fight gets them up again a minute later (content/questbattle/5061.lua) (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.)
local DAMAGE = 9999999

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, DAMAGE)
    -- (Retail's second effect: the action's own number as a combo's.)
    effects:execute_combo(0)

    return effects
end
