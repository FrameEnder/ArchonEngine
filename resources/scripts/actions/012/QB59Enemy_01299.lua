-- The Lominsan Way's quest battle (59): its first enemy's (BNpcBase 1769, layout 4295469) 2.5 s line from itself (the
-- Action sheet's area), aimed at the player whoever it's fighting (every cast: its timeline's `facing`), about every
-- 15 s from 38 s into the fight; 60 on the player the one time they stood in it (Bob's Chronofoil captures 44df02fc and c9145fdc, 2026-11-03, seven
-- runs). Its timeline's `recorded_amounts` scales it.
local POTENCY = 100
-- It knocks whoever it hits back the way it went: the Knockback sheet's row 1, as retail's ActionEffect had it after
-- the damage (`1F ... 01 00`); the client moves them, about 8 yalms.
local KNOCKBACK = 1

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))
    effects:knockback(KNOCKBACK)

    return effects
end
