-- Whispering Dawn (Lv. 20): healing over time for party members, 21 s.
-- NOTE: Scholar's Aetherflow and Faerie gauges and the faerie aren't implemented: the caster gives it.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 80
local DURATION = 21.0
local EFFECT_WHISPERING_DAWN = 315

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_over_time(EFFECT_WHISPERING_DAWN, DURATION, healAmount(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
