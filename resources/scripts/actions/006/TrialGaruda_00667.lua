-- Garuda's Mistral Shriek (667, the Howling Eye, 3.7-4.5 s, from the north, cast on herself): a magic hit on everyone
-- the monoliths don't shelter (action_areas.json's `sheltered_by`), and on the monoliths (1216 to players, 719 to a
-- monolith). From Bob's Chronofoil captures of 2026-11-08 (6a5244ec, 09ecf8e7, 6c495fc4); the hit is scaled to the
-- timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
