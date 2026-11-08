-- History Repeating's quest battle (86): the third wave's pair's hard hit (BNpcBase 2200; 389 on the player once,
-- Bob's Chronofoil captures of 2026-11-08), scaled to the timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
