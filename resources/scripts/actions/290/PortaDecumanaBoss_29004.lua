-- The Porta Decumana's bosses' 29004, their auto-attack: every 3.0 s for 145 to 253 to Duty Support's tank, 616 to a
-- player once (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run each.)
-- The timelines' `recorded_amounts` scale it.

local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, magicalDamage(player, POTENCY))

    return effects
end
