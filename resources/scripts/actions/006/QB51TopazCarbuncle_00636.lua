-- Storm, the Topaz Carbuncle's in Over the Rails (quest battle 51, BNpcBase 2056): a hit on its target that stunned it
-- (status 2; Bob's Kawari Recorder run of 2026-11-12, once: 4 on a gunner, which was stunned at the next sample.) How
-- long the stun lasts wasn't seen: 2 s here.
local POTENCY = 100
local STUN = 2
local STUN_SECONDS = 2.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(STUN, 0, STUN_SECONDS)

    return effects
end
