-- Painflare (Lv. 40): the target and enemies near it.
-- NOTE: Summoner's Arcanum, attunement and Aetherflow (the job gauge) and pets' attacks aren't implemented: it doesn't need or spend Aetherflow.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 220

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    return effects
end
