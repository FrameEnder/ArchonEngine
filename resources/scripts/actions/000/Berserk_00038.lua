-- Berserk (Lv. 6): 3 stacks guaranteeing critical direct hits for weaponskills, 15 s.
-- NOTE: whether the engine honours the guarantee depends on its critical hit rolls.
local EFFECT_BERSERK = 86
local STACKS = 3
local DURATION = 15.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_BERSERK, STACKS, DURATION)

    return effects
end
