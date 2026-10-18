-- Bio (Lv. 2): damage over time only.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local DOT_POTENCY = 20
local DURATION = 30.0
local EFFECT_BIO = 179

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage_over_time(EFFECT_BIO, DURATION, magicalDamage(player, classPotency(player, DOT_POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
