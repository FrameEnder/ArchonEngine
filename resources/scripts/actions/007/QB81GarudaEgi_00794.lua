-- Aerial Slash, Garuda-Egi's in Austerities of Wind (quest battle 81, BNpcBase 2209): instant, 5 yalms round where it
-- lands (the Action sheet's), twice, near the end of the lost run, 297 on the player (Bob's Kawari Recorder runs of
-- 2026-11-12; its timeline's recorded amount.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
