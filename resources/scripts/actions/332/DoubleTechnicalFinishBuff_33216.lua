-- Double Technical Finish Buff (Lv. 70): what the client sends after Double Technical Finish (retail: always straight after 16196, with the statuses it gave): Technical Finish (20 s) and Esprit for the party.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(DNC_EFFECT_TECHNICAL_FINISH, 0, 20.0)
    effects:gain_effect(DNC_EFFECT_TECHNICAL_ESPRIT, 0, 20.0)

    return effects
end
