-- The Lominsan Way's quest battle (59): its first enemy's (BNpcBase 1769, layout 4295469) 2.5 s line from itself (the
-- Action sheet's area), aimed at the one it's fighting (BNpcBase 1771), about every 15 s from 38 s into the fight; 60
-- on the player the one time they stood in it (Bob's Chronofoil captures 44df02fc and c9145fdc, 2026-11-03, seven
-- runs). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
