-- Radiant Finale (Lv. 90): 2, 4 or 6 % more damage for the party near (20 s) for one, two or three Codas (all spent); Radiant Encore Ready (30 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local coda = player:gauge("coda")
    local count = (coda & 1) + ((coda >> 1) & 1) + ((coda >> 2) & 1)
    effects:gain_effect(2964, count, 20.0)
    effects:gauge_set("coda", 0)
    if player.level >= 100 then
        effects:gain_effect_self(BRD_EFFECT_RADIANT_ENCORE_READY, count, 30.0)
    end

    return effects
end
