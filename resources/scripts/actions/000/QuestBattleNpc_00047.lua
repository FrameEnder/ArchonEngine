-- An action of the old marauder's kit that quest battle NPCs still use (History Repeating's marauders, quest battle
-- 86, BNpcBase 1649): a slashing hit (15, the timeline's recorded amount) that heals the user 4 (Bob's Chronofoil
-- captures of 2026-11-08: its effects, once). Retail's numbers, as fixed amounts.
local POTENCY = 100
local HEALED = 4

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))
    effects:heal_self(HEALED)

    return effects
end
