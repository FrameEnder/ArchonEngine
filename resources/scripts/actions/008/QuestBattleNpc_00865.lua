-- An action quest battle NPCs use (Big Trouble in Little Ala Mhigo's third-wave pugilists, BNpcBase 1729): one to three
-- of the other side caught, 126-243 (Bob's Chronofoil capture 20ef0e90, 2026-10-31, seven uses). What it does there
-- comes from the NPC's timeline (recorded_amounts), not from this potency.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
