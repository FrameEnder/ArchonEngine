-- Presence Of Mind (Lv. 30)
-- Cast times are reduced by 20% while active (recast and auto-attack delay reductions aren't implemented.)
DURATION = 15.0
SACRED_SIGHT_STACKS = 3
SACRED_SIGHT_DURATION = 30.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_PRESENCE_OF_MIND, 0, DURATION)

    -- "Enhanced Presence of Mind" grants Sacred Sight, which enables Glare IV
    if player:has_trait(TRAIT_ENHANCED_PRESENCE_OF_MIND) then
        effects:gain_effect_self(EFFECT_SACRED_SIGHT, SACRED_SIGHT_STACKS, SACRED_SIGHT_DURATION)
    end

    return effects
end
