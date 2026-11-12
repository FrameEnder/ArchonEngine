-- Gravity Force, the dark summoner's in Austerities of Earth (quest battle 80, BNpcBase 2206): a 3 s cast on the
-- ground, 6 yalms round (the Action sheet's, with its omen), 1067 on Y'mhitra both times with Heavy (status 14, param
-- 50; Bob's Kawari Recorder run of 2026-11-12.) How long the Heavy lasts wasn't seen: 10 s here.
local POTENCY = 100
local HEAVY = 14
local HEAVY_PARAM = 50
local HEAVY_SECONDS = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(HEAVY, HEAVY_PARAM, HEAVY_SECONDS)

    return effects
end
