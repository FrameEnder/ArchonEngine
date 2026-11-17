-- Axe in the Stone's big enemies' action (quest battle 25, BNpcBase 957): instant, every 6.4 to 8.4 s, a hit on its
-- target: 26 to 47 to the allies against its auto-attack's 18 to 26 (Bob's Chronofoil capture 0cafbf61 of 2026-11-17,
-- thirteen uses, four misses). Its timeline's `recorded_amounts` scales it.
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
