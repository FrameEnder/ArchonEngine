-- The Wanderers Minuet (Lv. 52): 2 % more critical hits for the party near, 45 s; Repertoire (up to three) for Pitch Perfect; Wanderer's Coda from 90. Ends the other songs.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    brdStartSong(effects, player, BRD_EFFECT_WANDERERS_MINUET, 4)

    return effects
end
