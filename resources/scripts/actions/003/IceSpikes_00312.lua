-- Trickster Imp. Gives itself Ice Spikes (status 198, param 100) for 20 seconds, as recorded on retail.
-- TODO: Ice Spikes should hurt (and slow?) whoever hits it in melee; for now it's only shown.
EFFECT_ICE_SPIKES = 198
DURATION = 20.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:gain_effect_self(EFFECT_ICE_SPIKES, 100, DURATION)

    return effects
end
