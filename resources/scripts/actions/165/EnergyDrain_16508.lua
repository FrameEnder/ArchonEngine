-- Energy Drain (Lv. 10), Arcanist/Summoner.
-- Fills Aetherflow (two stacks, for Fester and Painflare). Summoner's Arcanum, attunement, Further Ruin and pets'
-- attacks aren't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 200

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    effects:gain_aetherflow()

    return effects
end
