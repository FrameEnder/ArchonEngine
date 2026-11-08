-- The plumes' blast (665, the Howling Eye, round themselves, instant): a magic hit (705 to a monolith, 191-206 to Duty
-- Support's members); the plume falls 2.3 s later, the director's. From Bob's Chronofoil captures of 2026-11-08
-- (6a5244ec, 09ecf8e7, 6c495fc4); the hit is scaled to the timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
