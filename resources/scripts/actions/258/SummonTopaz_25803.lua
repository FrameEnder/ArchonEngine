-- Summon Topaz (Lv. 15): the carbuncle's Glittering attack, done here as the summoner's.
-- NOTE: Summoner's Arcanum, attunement and Aetherflow (the job gauge) and pets' attacks aren't implemented: no attunement for Gemshine is given.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 400

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
