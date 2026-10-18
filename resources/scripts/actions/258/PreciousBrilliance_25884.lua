-- Precious Brilliance (Lv. 26): the attuned element's area spell.
-- NOTE: Summoner's Arcanum, attunement and Aetherflow (the job gauge) and pets' attacks aren't implemented: it always does the fire (Ruby) version, whose potency is a guess.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
