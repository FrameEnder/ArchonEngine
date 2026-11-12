-- Shockwave, Garuda-Egi's (and Chirada-Egi's and Suparna-Egi's) in Austerities of Wind (quest battle 81, BNpcBase
-- 2209, 2212, 2213): a 2 s cast, 3 yalms round itself (the Action sheet's, with its omen), every 10.2-19.1 s. It hit
-- nobody in Bob's Kawari Recorder runs of 2026-11-12, so how hard it hits isn't recorded: an attack's worth.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
