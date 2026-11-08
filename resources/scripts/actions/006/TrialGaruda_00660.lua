-- Garuda's Aerial Blast (660, the Howling Eye, 3.7 s, from the middle): a magic hit on everyone (827 on a player). From
-- Bob's Chronofoil captures of 2026-11-08 (6a5244ec, 09ecf8e7, 6c495fc4); the hit is scaled to the timeline's recorded
-- amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
