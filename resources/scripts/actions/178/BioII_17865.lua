-- Bio II (Lv. 26, replaces Bio): damage over time only.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local DOT_POTENCY = 40
local DURATION = 30.0
local EFFECT_BIO_II = 189

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(EFFECT_BIO_II, DURATION, magicalDamage(player, classPotency(player, DOT_POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
