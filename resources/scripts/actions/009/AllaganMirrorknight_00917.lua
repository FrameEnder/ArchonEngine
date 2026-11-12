-- Gust, the Allagan mirrorknights' (BNpcBase 1948, Allagan Attire): magic on the ground, 3 yalms round (the Action
-- sheet's). Bob's Kawari Recorder export of 2026-11-12 saw it cast three times and hit nobody, so how hard it hits isn't
-- recorded: an auto-attack's worth.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
