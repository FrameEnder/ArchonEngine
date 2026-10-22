-- Painflare (Lv. 40): the target and enemies near it.
-- Spends one Aetherflow stack (the client only offers it with one; Energy Drain fills them.) Summoner's Arcanum,
-- attunement and pets' attacks aren't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 220

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    effects:spend_aetherflow()

    return effects
end
