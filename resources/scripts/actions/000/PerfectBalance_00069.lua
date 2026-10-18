-- Perfect Balance (Lv. 50): 3 stacks, each allowing a form's weaponskill without the form, 20 s.
local EFFECT_PERFECT_BALANCE = 110
local STACKS = 3
local DURATION = 20.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_PERFECT_BALANCE, STACKS, DURATION)

    return effects
end
