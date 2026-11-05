-- Viscous Discharge (1166): the slug in Seer Folly's quest battle 75 (BNpcBase 1677), instant, 6 yalms round itself,
-- once 19.4 s after it came: no damage, Silence (7) on everyone it caught (you and A-Ruhn-Senna; ActionEffect
-- `0E 07`), which wore off 10.3 s after it went on (Bob's Chronofoil capture 8d550e33, 2026-11-05.)
local EFFECT_SILENCE = 7
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(EFFECT_SILENCE, 0, DURATION)

    return effects
end
